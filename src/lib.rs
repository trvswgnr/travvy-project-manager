//! # Travvy's Project Manager Library (`tpm_lib`)
//!
//! `tpm_lib` serves as the core library for Travvy's Project Manager (`tpm`),
//! a command-line utility designed to streamline project management tasks such
//! as opening, creating, listing, editing, and deleting project entries.
//!
//! This library provides abstractions and utility functions to facilitate user
//! interaction and data manipulation.
//!
//! ## Primary Entities
//!
//! - [`Project`]: Represents a single project with attributes like name and path.
//! - [`Action`]: Enumerates the different actions that can be performed on projects.
//! - [`DynErr`]: Represents dynamic errors that can occur within the application.
//! - [`Dialogue<'a>`]: Abstraction for handling interactive dialogues.
//!
//! ## Features
//!
//! - **Interactive Mode**: A command-line interface for interactively managing projects.
//! - **Project Management**: Functions for adding, editing, and deleting project entries.
//! - **File Operations**: Functions for loading and saving project data from and to disk.
//!
//! ## Dependencies
//!
//! - [clap]- For command-line argument parsing.
//! - [serde]
//! - [serde_json] - For serialization and deserialization.
//! - [dialoguer] - For constructing interactive command-line interfaces.
//! - [lazy_static] - For lazily-evaluated statics.
//!
//! ## Usage
//!
//! This library is primarily intended to be used by the `tpm` binary, but it
//! exposes public interfaces that could be utilized in custom extensions or
//! other binaries.
//!
//! ```no_run
//! use tpm_lib::{load_projects, Action};
//!
//! let projects = load_projects();
//! // Custom logic here
//! ```
//!
//! For more examples and usage guidelines, refer to the
//! [README.md](https://github.com/trvswgnr/travvy-project-manager#readme).
//!
//! ## Contribute
//!
//! For contributing guidelines, please refer to the
//! [README.md](https://github.com/trvswgnr/travvy-project-manager#contributing).
//!
//! ## License
//!
//! This project is licensed under the MIT License - see the
//! [LICENSE](https://github.com/trvswgnr/travvy-project-manager/blob/main/LICENSE)
//! file for details.
//!
//! [`Project`]: crate::Project
//! [`Action`]: crate::Action
//! [`DynErr`]: crate::DynErr
//! [`Dialogue<'a>`]: crate::Dialogue
//! [clap]: https://crates.io/crates/clap
//! [serde]: https://crates.io/crates/serde
//! [serde_json]: https://crates.io/crates/serde_json
//! [dialoguer]: https://crates.io/crates/dialoguer
//! [lazy_static]: https://crates.io/crates/lazy_static
//!

use clap::{App, Arg, ArgMatches, SubCommand, ValueHint};
use dialoguer::{console, theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    env,
    error::Error,
    ffi::OsString,
    fmt,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{self, Command},
    sync::Mutex,
    time::{Duration, SystemTime},
};

lazy_static! {
    /// A thread-safe lazily-evaluated static variable that holds a list of projects.
    static ref PROJECTS: Mutex<Vec<Project>> = Mutex::new(load_projects_from_disk());
}

/// A shared resource that tracks the number of visits to the home interface.
pub static mut HOME_INTERFACE_VISITS: Mutex<usize> = Mutex::new(0);

/// Parses command line arguments and returns a struct containing the parsed values.
///
/// # Arguments
///
/// * `args` - Any value that can be converted into an iterator that yields
///   values that can be converted to `OsString`.
///            Usually this is `std::env::args()`.
///
/// # Examples
///
/// ```
/// use clap::ArgMatches;
/// use tpm_lib::get_matches;
///
/// let args = vec!["tpm", "add", "foo", "bar"];
/// let matches = get_matches(args);
///
/// assert_eq!(matches.subcommand_name(), Some("add"));
/// let add_matches = matches.subcommand_matches("add").unwrap();
/// assert_eq!(add_matches.value_of("project_name"), Some("foo"));
/// assert_eq!(add_matches.value_of("project_path"), Some("bar"));
/// assert_eq!(add_matches.value_of("name"), None);
/// ```
pub fn get_matches<I, T>(args: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let about = "\n".to_string() + ABOUT;
    let app = App::new(
        WELCOME_SCREEN
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .version(VERSION)
    .long_version(VERSION)
    .about(about.as_str())
    .arg(
        Arg::with_name("completions")
            .long("completions")
            .value_name("SHELL")
            .help("Installs shell completions for the current user")
            .forbid_empty_values(false)
            .min_values(0)
            .possible_values(VALID_SHELLS)
            .value_hint(ValueHint::Other)
            .required(false),
    )
    .subcommand(
        SubCommand::with_name("add")
            .about("Add a new project")
            .arg(Arg::from_usage("<project_name> 'Project name'").required(false))
            .arg(Arg::from_usage("<project_path> 'Project path'").required(false))
            .arg(
                Arg::with_name("name")
                    .short('n')
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("path")
                    .short('p')
                    .takes_value(true)
                    .required(false),
            ),
    )
    .subcommand(SubCommand::with_name("list").about("List all projects"))
    .subcommand(
        SubCommand::with_name("delete")
            .about("Delete a project")
            .arg(Arg::from_usage("<project_name> 'Project name'").required(false))
            .arg(
                Arg::with_name("name")
                    .short('n')
                    .takes_value(true)
                    .required(false),
            ),
    )
    .subcommand(
        SubCommand::with_name("edit")
            .about("Edit a project")
            .arg(Arg::from_usage("<project_name> 'Project name'").required(false))
            .arg(
                Arg::with_name("name")
                    .short('n')
                    .takes_value(true)
                    .required(false),
            ),
    )
    .subcommand(
        SubCommand::with_name("open")
            .about("Open a project")
            .arg(
                Arg::from_usage("<project_name> 'Project name'")
                    .required(false)
                    .value_hint(ValueHint::Other),
            )
            .arg(
                Arg::with_name("name")
                    .short('n')
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("editor")
                    .help("Open in editor instead of terminal")
                    .short('e')
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::with_name("replace")
                    .help("Replace current editor with project, instead of opening in a new window")
                    .short('r')
                    .takes_value(false)
                    .required(false)
                    .requires("editor"),
            ),
    )
    .subcommand(
        SubCommand::with_name("new")
            .about("Create a new project")
            .arg(Arg::from_usage("<project_name> 'Project name'").required(false))
            .arg(
                Arg::with_name("name")
                    .short('n')
                    .takes_value(true)
                    .required(false),
            ),
    )
    .get_matches_from(args);

    let m = app.subcommand_name();
    println!("m: {:?}", m);
    let name = app.subcommand().unwrap().1.value_of("project_name");
    println!("name: {:?}", name);
    app
}

pub fn handler(matches: &ArgMatches) {
    if matches.args_present() && matches.contains_id("completions") {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Install completions?")
            .default(true)
            .interact()
            .unwrap_or_else(|e| panic!("Error: {}", e));
        if !confirmed {
            exit_with(Ok("Goodbye!"));
        }
        let shell = matches
            .value_of("completions")
            .map_or_else(get_current_shell, |s| s.to_string());
        gen_completions(&shell);
    }

    match matches.subcommand().unwrap_or(("", &ArgMatches::default())) {
        ("add", add_matches) => {
            let name = add_matches
                .value_of("name")
                .unwrap_or(add_matches.value_of("project_name").unwrap_or(""));
            let path = add_matches
                .value_of("path")
                .unwrap_or(add_matches.value_of("project_path").unwrap_or(""));
            if name.is_empty() && path.is_empty() {
                return show_add_project_interface();
            }
            add_project(name, path);
        }
        ("list", _) => {
            let projects = load_projects();
            if projects.is_empty() {
                return select_no_projects_found();
            }
            // term height without using crates
            let term_height = console::Term::stdout().size().0;
            Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Your projects")
                .items(&projects)
                .default(0)
                .max_length(term_height as usize - 1)
                .interact_opt()
                .unwrap_or(None);
        }
        ("delete", delete_matches) => {
            let name = delete_matches
                .value_of("name")
                .unwrap_or(delete_matches.value_of("project_name").unwrap_or(""));
            if name.is_empty() {
                show_select_projects_interface(Action::Delete, Some("Select projects to delete"));
                return;
            }
            delete_project(name);
        }
        ("edit", edit_matches) => {
            let name = edit_matches
                .value_of("name")
                .unwrap_or(edit_matches.value_of("project_name").unwrap_or(""));

            if name.is_empty() {
                show_select_projects_interface(Action::Edit, Some("Select a project to edit"));
                return;
            }
            edit_project(name);
        }
        ("open", open_matches) => {
            let name = open_matches
                .value_of("name")
                .unwrap_or(open_matches.value_of("project_name").unwrap_or(""));
            if name.is_empty() {
                show_select_projects_interface(Action::Open, Some("Select a project to open"));
                return;
            }

            let open_action = if open_matches.is_present("editor") {
                OpenAction::OpenInEditor
            } else {
                OpenAction::OpenInTerminal
            };

            let replace_editor = open_matches.is_present("replace");

            open_project(name, open_action, replace_editor);
        }
        ("new", new_matches) => {
            let name = new_matches
                .value_of("name")
                .unwrap_or(new_matches.value_of("project_name").unwrap_or(""));
            if name.is_empty() {
                return show_new_project_interface();
            }
            new_project(name, "");
        }
        _ => show_home_interface("What would you like to do?"),
    }
}

/// Increments the number of visits to the home interface by one.
///
/// # Safety
///
/// This function uses unsafe code to access a shared resource (`HOME_INTERFACE_VISITS`)
/// without any form of synchronization. It is the caller's responsibility to ensure
/// that this function is only called from a single thread at a time, or to provide
/// appropriate synchronization mechanisms to prevent data races.
pub fn increment_visits() {
    unsafe {
        let mut visits = HOME_INTERFACE_VISITS.lock().unwrap();
        *visits += 1;
    }
}

pub fn get_visits() -> usize {
    unsafe {
        let visits = HOME_INTERFACE_VISITS.lock().unwrap();
        *visits
    }
}

/// the app name, used everywhere
pub const APP_NAME: &str = "tpm";
pub const VALID_SHELLS: [&str; 2] = ["bash", "zsh"];
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

pub const WELCOME_SCREEN: &str = r"
                                  __
    ____  _________  _____  _____/ /______
   / __ \/ ___/ __ \/ / _ \/ ___/ __/ ___/
  / /_/ / /  / /_/ / /  __/ /__/ /__\__ \
 / .___/_/   \____/ /\___/\___/\___/____/
/_/            /___/
";

pub enum DynErr {
    String(String),
    Io(io::Error),
    Serde(serde_json::Error),
    Std(Box<dyn Error>),
}

impl From<String> for DynErr {
    fn from(err: String) -> Self {
        DynErr::String(err)
    }
}

impl From<&str> for DynErr {
    fn from(err: &str) -> Self {
        DynErr::String(err.to_string())
    }
}

impl From<OsString> for DynErr {
    fn from(err: OsString) -> Self {
        DynErr::String(err.into_string().unwrap())
    }
}

impl From<io::Error> for DynErr {
    fn from(err: io::Error) -> Self {
        DynErr::Io(err)
    }
}

impl From<serde_json::Error> for DynErr {
    fn from(err: serde_json::Error) -> Self {
        DynErr::Serde(err)
    }
}

impl From<Box<dyn Error>> for DynErr {
    fn from(err: Box<dyn Error>) -> Self {
        DynErr::Std(err)
    }
}

impl fmt::Display for DynErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DynErr::String(err) => write!(f, "{}", err),
            DynErr::Io(err) => write!(f, "{}", err),
            DynErr::Serde(err) => write!(f, "{}", err),
            DynErr::Std(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Open,
    Delete,
    Edit,
}

/// gets the current shell from the SHELL environment variable
///
/// if shell is not in VALID_SHELLS, exits with an error
pub fn get_current_shell() -> String {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let shell = shell.split('/').last().unwrap_or("sh");
    // check if shell is in VALID_SHELLS
    if VALID_SHELLS.contains(&shell) {
        println!("Detected shell: {shell}");
        return shell.to_string();
    }
    let msg = format!(
        "Invalid shell: {shell}. Valid shells: {valid_shells}",
        valid_shells = VALID_SHELLS.join(", ")
    );
    exit_with(Err(msg.into()));
}

pub fn get_path_to_shell_profile(shell: &str) -> PathBuf {
    let home_dir = PathBuf::from(env::var("HOME").unwrap_or("/".to_string()));
    match shell {
        "bash" => home_dir.join(".bash_profile"),
        "zsh" => home_dir.join(".zshrc"),
        _ => exit_with(Err("Invalid shell".into())),
    }
}

pub fn gen_completions(shell: &str) {
    let script = r#"
__tpm() {
    local cur
    local prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    case ${COMP_CWORD} in
    1)
        COMPREPLY=($(compgen -W "open add edit delete new" -- ${cur}))
        ;;
    2)
        case ${prev} in
        open | edit | delete)
            COMPREPLY=($(compgen -W "$(cat {%config_dir%}/project_names.txt)" -- ${cur}))
            ;;
        *)
            ;;
        esac
        ;;
    esac
}

complete -F __tpm {%app_name%}
"#;

    let config_dir = get_config_dir()
        .canonicalize()
        .unwrap_or_else(|e| exit_with(Err(e.into())));
    let config_dir_str = config_dir
        .to_str()
        .unwrap_or_else(|| exit_with(Err("Problem converting config dir to string".into())));
    let script = script
        .replace("{%app_name%}", APP_NAME)
        .replace("{%config_dir%}", config_dir_str);

    let completions_filename = format!("{}_completions.sh", APP_NAME);
    let completions_file = config_dir.join(&completions_filename);
    let mut file = File::create(&completions_file).unwrap_or_else(|e| exit_with(Err(e.into())));
    file.write_all(script.as_bytes())
        .unwrap_or_else(|e| exit_with(Err(e.into())));

    let shell_profile = get_path_to_shell_profile(shell);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&shell_profile)
        .unwrap_or_else(|e| exit_with(Err(e.into())));
    let script = format!(
        "\n# {} completions\nsource {}\n",
        APP_NAME,
        completions_file.to_str().unwrap()
    );

    // check if the file already contains the script
    let mut contents = String::new();
    let mut read_file = File::open(&shell_profile).unwrap_or_else(|e| exit_with(Err(e.into())));
    read_file
        .read_to_string(&mut contents)
        .unwrap_or_else(|e| exit_with(Err(e.into())));

    // check if contents contains `source path/to/{APP_NAME}_completions.sh`
    if contents
        .lines()
        .any(|line| line.contains("source") && line.contains(&completions_filename))
    {
        let msg = format!(
            "Completions already installed for {:?} in {:?}",
            shell,
            shell_profile.to_str().unwrap()
        );
        exit_with(Ok(&msg));
    }

    file.write_all(script.as_bytes())
        .unwrap_or_else(|e| exit_with(Err(e.into())));

    let msg = format!(
        "Completions installed for {} in {:?}",
        shell,
        shell_profile.to_str().unwrap()
    );
    exit_with(Ok(&msg));
}

pub fn exit_with(result: Result<&str, DynErr>) -> ! {
    match result {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{}", msg);
            }
            process::exit(0);
        }
        Err(msg) => {
            if !msg.to_string().is_empty() {
                println!("{}", msg);
            }
            process::exit(1);
        }
    }
}

pub fn show_new_project_interface() {
    let name = Input::<String>::new()
        .with_prompt("Project name")
        .interact_text()
        .unwrap_or_default();

    if name.trim().is_empty() {
        println!("Name cannot be empty");
        return show_new_project_interface();
    }

    if project_already_exists(name.trim()) {
        println!("A project with that name already exists");
        return show_new_project_interface();
    }

    let home_dir = PathBuf::from(env::var("HOME").unwrap_or("/".to_string()));
    let project_folder = home_dir.join("projects");
    let name_normalized = name
        .trim()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let default_path_string = project_folder
        .join(name_normalized)
        .to_str()
        .unwrap()
        .to_string();
    let path = Input::<String>::new()
        .with_prompt("Project path")
        .default(default_path_string)
        .interact_text()
        .unwrap_or_default();

    if path.trim().is_empty() {
        println!("Path cannot be empty");
        return show_new_project_interface();
    }

    if project_already_exists(path.trim()) {
        println!("A project with that path already exists");
        return show_new_project_interface();
    }

    new_project(name.trim(), path.trim());
}

pub fn new_project(name: &str, path: &str) {
    if name.is_empty() {
        println!("Name cannot be empty");
        return show_new_project_interface();
    }
    let mut projects = load_projects();
    let name_normalized = name
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let home_dir = PathBuf::from(env::var("HOME").unwrap_or("/".to_string()));
    let project_folder = home_dir.join("projects");
    let default_path_string = project_folder
        .join(name_normalized)
        .to_str()
        .unwrap()
        .to_string();
    let path_string = if path.is_empty() {
        default_path_string
    } else {
        path.to_string()
    };
    let path = PathBuf::from(path_string.clone())
        .canonicalize()
        .unwrap_or_else(|_| create_path_with_parent_dirs(&path_string));
    if path.exists() {
        println!("A project with that path already exists");
        println!("Path: {:?}", path);
        return show_new_project_interface();
    }
    fs::create_dir(&path).unwrap();
    let mut project = Project {
        name: name.to_string(),
        path: path.to_str().unwrap().to_string(),
        last_opened: Duration::from_secs(0),
    };
    project.set_last_opened();
    if project_already_exists(&project.name) {
        return show_overwrite_project_interface(&project);
    }
    projects.push(project.clone());
    save_projects(&projects);
    open_project(&project.name, OpenAction::OpenInTerminal, false);
}

pub fn create_path_with_parent_dirs(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    let parent = path.parent();
    if parent.is_none() {
        return path;
    }
    let parent = parent.unwrap();
    if !parent.exists() {
        create_path_with_parent_dirs(parent.to_str().unwrap());
    }
    path
}

pub fn show_home_interface(prompt: &str) {
    increment_visits();
    let projects = load_projects();
    let mut project_names = Vec::new();
    for project in projects.iter() {
        project_names.push(project.name.as_str());
    }

    let prompt = if get_visits() == 1 {
        format!("{}\n{}", WELCOME_SCREEN, "Press enter to continue")
    } else {
        prompt.to_string()
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&[
            "Open project",
            "Add project",
            "Edit project",
            "Delete projects",
            "New project",
            "Quit (Esc)",
        ])
        .default(0)
        .interact_opt()
        .unwrap_or(None);

    if selection.is_none() {
        return quit();
    }

    let selection = selection.unwrap();

    match selection {
        0 => show_select_projects_interface(Action::Open, Some("Select a project to open")),
        1 => show_add_project_interface(),
        2 => show_select_projects_interface(Action::Edit, Some("Select a project to edit")),
        3 => show_select_projects_interface(Action::Delete, Some("Select projects to delete")),
        4 => show_new_project_interface(),
        _ => quit(),
    }
}

pub fn select_no_projects_found() {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("No projects found")
        .items(&["Add project", "Quit"])
        .default(0)
        .interact()
        .unwrap_or(1);
    match selection {
        0 => show_add_project_interface(),
        _ => quit(),
    }
}

pub fn quit<T>() -> T {
    println!("Goodbye!");
    process::exit(0);
}

trait IntoString {
    fn into_string(self) -> Result<String, DynErr>;
}

impl IntoString for OsString {
    fn into_string(self) -> Result<String, DynErr> {
        self.into_string().map_err(|err| err.into())
    }
}

pub fn show_add_project_interface() {
    let current_dir = env::current_dir().unwrap();
    let default_name = current_dir
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let default_path = current_dir.to_str().unwrap().to_string();
    let name = Input::<String>::new()
        .with_prompt("Project name")
        .default(default_name)
        .interact_text()
        .unwrap_or_default();
    let path = Input::<String>::new()
        .with_prompt("Project path")
        .default(default_path)
        .interact_text()
        .unwrap_or_default();
    if name.is_empty() || path.is_empty() {
        println!("Name and path cannot be empty");
        return show_add_project_interface();
    }
    add_project(name.as_str(), path.as_str());
}

pub enum Dialogue<'a> {
    Select(Select<'a>),
    MultiSelect(MultiSelect<'a>),
    // Confirm(Confirm<'a>),
    // Input(Input<'a, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Default)]
pub struct Project {
    name: String,
    path: String,
    last_opened: Duration,
}

impl Project {
    fn set_last_opened(&mut self) {
        self.last_opened = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.path)
    }
}

pub fn load_projects_from_disk() -> Vec<Project> {
    let mut file = open_projects_file(true, false, false);
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let projects_set: HashSet<Project> = serde_json::from_str(&json).unwrap_or_default();
    let mut projects: Vec<Project> = projects_set.into_iter().collect();
    // sort by last opened (most recent first)
    projects.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    projects
}

pub fn load_projects() -> Vec<Project> {
    let projects = PROJECTS.lock().unwrap();
    projects.to_vec()
}

pub fn save_projects(projects: &[Project]) {
    let mut file = File::create(get_config_dir().join("projects.json")).unwrap();
    let json = serde_json::to_string_pretty(&projects).unwrap();
    file.write_all(json.as_bytes()).unwrap();
    *PROJECTS.lock().unwrap() = projects.to_vec();

    // also save a list of project names to a file for use in bash completion
    let mut file = File::create(get_config_dir().join("project_names.txt")).unwrap();
    let mut names = Vec::new();
    for project in projects {
        names.push(project.name.as_str());
    }
    let names = names.join("\n");
    file.write_all(names.as_bytes()).unwrap();
}

pub fn add_project(name: &str, path: &str) {
    let mut projects = load_projects();
    let default_path = env::current_dir().unwrap();
    let default_name = default_path.file_name().unwrap().to_str().unwrap();
    let name = if name.is_empty() { default_name } else { name };
    let path = if path.is_empty() {
        PathBuf::from(default_path.to_str().unwrap())
    } else {
        PathBuf::from(path).canonicalize().unwrap_or_else(|e| {
            exit_with(Err(e.into()));
        })
    };
    let mut project = Project {
        name: name.to_string(),
        path: path.to_str().unwrap().to_string(),
        last_opened: Duration::from_secs(0),
    };
    project.set_last_opened();
    if project_already_exists(&project.name) {
        return show_overwrite_project_interface(&project);
    }
    projects.push(project.clone());
    save_projects(&projects);
}

pub fn show_overwrite_project_interface(project: &Project) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Project {} already exists. Overwrite?",
            project.name
        ))
        .items(&["Yes", "No", "Back", "Quit"])
        .default(0)
        .interact()
        .unwrap_or(1);
    match selection {
        0 => {
            // confirm overwrite
            let selection = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Overwrite project {}", project.name))
                .default(false)
                .interact()
                .unwrap();
            if selection {
                // overwrite
                let mut projects = load_projects();
                projects.retain(|p| p != project);
                projects.push(project.clone());
                save_projects(&projects);
            }
            show_home_interface("What would you like to do?");
        }
        1 => show_add_project_interface(),
        2 => show_home_interface("What would you like to do?"),
        _ => quit(),
    }
}

pub fn project_already_exists(name_or_path: &str) -> bool {
    let projects = load_projects();
    projects
        .iter()
        .any(|p| p.name == name_or_path || p.path == name_or_path)
}

pub fn show_select_projects_interface(action: Action, prompt: Option<&str>) {
    let projects = load_projects();

    if projects.is_empty() {
        return select_no_projects_found();
    }

    let project_names = projects
        .iter()
        .map(|project| project.name.as_str())
        .collect::<Vec<_>>();

    let theme = ColorfulTheme::default();

    let dialogue = match action {
        Action::Delete => Dialogue::MultiSelect(
            MultiSelect::with_theme(&theme)
                .with_prompt(prompt.unwrap_or("Select a project"))
                .items(&project_names)
                .max_length(5),
        ),
        _ => Dialogue::Select(
            Select::with_theme(&theme)
                .with_prompt(prompt.unwrap_or("Select a project"))
                .items(&project_names)
                .max_length(5),
        ),
    };

    let selections = match dialogue {
        Dialogue::Select(select) => select
            .default(0)
            .interact_opt()
            .unwrap_or_default()
            .map(|selection| vec![selection]),
        Dialogue::MultiSelect(multi_select) => multi_select.interact_opt().unwrap_or_default(),
    };

    if selections.is_none() || selections.as_ref().unwrap().is_empty() {
        show_home_interface("What would you like to do?");
        return;
    }

    let selections = selections.unwrap();

    if selections.is_empty() {
        println!("No project selected");
        return quit();
    }

    let mut selected_projects = Vec::new();
    for selection in selections {
        selected_projects.push(projects[selection].clone());
    }

    match action {
        Action::Open => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Open project in")
                .items(&["Terminal", "Editor", "Back", "Quit"])
                .default(0)
                .interact_opt()
                .unwrap_or_else(|e| panic!("Error: {}", e));

            if selection.is_none() {
                return show_select_projects_interface(Action::Open, None);
            }

            let selection = selection.unwrap();
            match selection {
                0 => {
                    let project = &selected_projects[0];
                    open_project(&project.name, OpenAction::OpenInTerminal, false);
                }
                1 => {
                    for project in selected_projects {
                        open_project(&project.name, OpenAction::OpenInEditor, false);
                    }
                }
                2 => {
                    show_select_projects_interface(Action::Open, None);
                }
                3 => quit(),
                _ => {}
            }
        }
        Action::Delete => {
            let also_delete_dir = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Also delete project directory?")
                .default(false)
                .interact()
                .unwrap();
            delete_projects(
                &selected_projects
                    .iter()
                    .map(|project| project.name.as_str())
                    .collect::<Vec<_>>(),
                also_delete_dir,
            );
        }
        Action::Edit => {
            for project in selected_projects {
                edit_project(&project.name);
            }
        }
    }
}

pub fn delete_project(name: &str) {
    let mut projects = load_projects();
    projects.retain(|project| project.name != name);
    save_projects(&projects);
}

pub fn delete_projects(names: &[&str], also_delete_dir: bool) {
    let mut projects = load_projects();
    if also_delete_dir {
        for name in names {
            let project = projects
                .iter()
                .find(|project| project.name == *name)
                .unwrap();
            fs::remove_dir_all(&project.path).unwrap_or_else(|_| {
                println!("Failed to delete project directory: {}", project.path)
            });
        }
    }
    projects.retain(|project| !names.contains(&project.name.as_str()));
    save_projects(&projects);
}

/// Shows an interface for editing a project and saves the changes.
pub fn edit_project(name: &str) {
    let mut projects = load_projects();
    if let Some(project) = projects.iter_mut().find(|project| project.name == name) {
        let new_name = Input::<String>::new()
            .with_prompt("Project name")
            .default(project.name.clone())
            .interact_text()
            .unwrap_or_default();
        let new_path = Input::<String>::new()
            .with_prompt("Project path")
            .default(project.path.clone())
            .interact_text()
            .unwrap();
        project.name = new_name;
        project.path = new_path;
        save_projects(&projects);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenAction {
    /// Open the project in the terminal (cd into the project folder)
    OpenInTerminal,
    /// Open the project in the default editor
    OpenInEditor,
}

pub fn open_project(name: &str, open_action: OpenAction, replace_editor: bool) {
    let mut projects = load_projects();
    if let Some((i, project)) = projects
        .clone()
        .iter_mut()
        .enumerate()
        .find(|(_, project)| project.name == name)
    {
        projects[i].set_last_opened();
        save_projects(&projects);
        match open_action {
            OpenAction::OpenInTerminal => {
                change_directory(&project.path).unwrap();
            }
            OpenAction::OpenInEditor => {
                open_in_editor(&project.path, replace_editor).unwrap();
            }
        }
    } else {
        println!("Project not found");
    }
}

pub fn change_directory(new_dir: &str) -> io::Result<()> {
    let path = Path::new(&new_dir);
    if path.exists() && path.is_dir() {
        env::set_current_dir(path)?;
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Command::new(shell).status()?;
    } else {
        eprintln!("cd: {}: No such file or directory", new_dir);
    }

    Ok(())
}

pub fn open_in_editor(path: &str, replace_editor: bool) -> io::Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(&editor)
        .arg(path)
        .arg(if replace_editor && editor == "code" {
            "--reuse-window"
        } else {
            ""
        })
        .status()?;
    Ok(())
}

pub fn get_config_dir() -> PathBuf {
    // check if a .config folder exists in the home directory
    let home_dir = dirs::home_dir().unwrap();
    let xdg_config_dir = home_dir.join(".config");
    let base_dir = if xdg_config_dir.exists() {
        xdg_config_dir
    } else {
        // use the home directory
        home_dir
    };
    let config_dir = base_dir.join(APP_NAME);
    if !config_dir.exists() {
        fs::create_dir(&config_dir).unwrap();
    }

    config_dir
}

pub fn open_projects_file(read: bool, write: bool, create: bool) -> File {
    let config_dir = get_config_dir();
    let projects_file = config_dir.join("projects.json");
    // if the file doesn't exist, create it
    if !projects_file.exists() {
        File::create(&projects_file).unwrap();
    }
    fs::OpenOptions::new()
        .read(read)
        .write(write)
        .create(create)
        .open(projects_file)
        .unwrap()
}