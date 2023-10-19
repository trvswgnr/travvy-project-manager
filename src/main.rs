use clap::{App, Arg, SubCommand};
use dialoguer::{console, theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

lazy_static! {
    static ref PROJECTS: Mutex<Vec<Project>> = Mutex::new(load_projects_from_disk());
}

// a global counter for home many times we've visited the home interface
static mut HOME_INTERFACE_VISITS: Mutex<usize> = Mutex::new(0);

fn increment_visits() {
    unsafe {
        let mut visits = HOME_INTERFACE_VISITS.lock().unwrap();
        *visits += 1;
    }
}

fn get_visits() -> usize {
    unsafe {
        let visits = HOME_INTERFACE_VISITS.lock().unwrap();
        *visits
    }
}

const WELCOME_SCREEN: &str = r"
                                  __
    ____  _________  _____  _____/ /______
   / __ \/ ___/ __ \/ / _ \/ ___/ __/ ___/
  / /_/ / /  / /_/ / /  __/ /__/ /__\__ \
 / .___/_/   \____/ /\___/\___/\___/____/
/_/            /___/
";

enum DynErr {
    String(String),
    Io(io::Error),
    Serde(serde_json::Error),
    Std(Box<dyn std::error::Error>),
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

impl From<std::ffi::OsString> for DynErr {
    fn from(err: std::ffi::OsString) -> Self {
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

impl From<Box<dyn std::error::Error>> for DynErr {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        DynErr::Std(err)
    }
}

impl std::fmt::Display for DynErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DynErr::String(err) => write!(f, "{}", err),
            DynErr::Io(err) => write!(f, "{}", err),
            DynErr::Serde(err) => write!(f, "{}", err),
            DynErr::Std(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Open,
    Delete,
    Edit,
}
const VERSION: &str = env!("CARGO_PKG_VERSION");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let about = "\n".to_string() + ABOUT;
    let matches = App::new(
        WELCOME_SCREEN
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .version(VERSION)
    .long_version(VERSION)
    .about(about.as_str())
    .subcommand(
        SubCommand::with_name("add")
            .about("Add a new project")
            .arg(Arg::from_usage("<project_name> 'Project name'").required(false))
            .arg(Arg::from_usage("<project_path> 'Project path'").required(false))
            .arg(
                Arg::with_name("name")
                    .short("n")
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("path")
                    .short("p")
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
                    .short("n")
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
                    .short("n")
                    .takes_value(true)
                    .required(false),
            ),
    )
    .subcommand(
        SubCommand::with_name("open")
            .about("Open a project")
            .arg(Arg::from_usage("<project_name> 'Project name'").required(false))
            .arg(
                Arg::with_name("name")
                    .short("n")
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("editor")
                    .help("Open in editor instead of terminal")
                    .short("e")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::with_name("replace")
                    .help("Replace current editor with project, instead of opening in a new window")
                    .short("r")
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
                    .short("n")
                    .takes_value(true)
                    .required(false),
            ),
    )
    .get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
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
        ("list", Some(_)) => {
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
        ("delete", Some(delete_matches)) => {
            let name = delete_matches
                .value_of("name")
                .unwrap_or(delete_matches.value_of("project_name").unwrap_or(""));
            if name.is_empty() {
                show_select_projects_interface(Action::Delete, Some("Select projects to delete"));
                return;
            }
            delete_project(name);
        }
        ("edit", Some(edit_matches)) => {
            let name = edit_matches
                .value_of("name")
                .unwrap_or(edit_matches.value_of("project_name").unwrap_or(""));

            if name.is_empty() {
                show_select_projects_interface(Action::Edit, Some("Select a project to edit"));
                return;
            }
            edit_project(name);
        }
        ("open", Some(open_matches)) => {
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
        ("new", Some(new_matches)) => {
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

fn show_new_project_interface() {
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

fn new_project(name: &str, path: &str) {
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
    std::fs::create_dir(&path).unwrap();
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

fn create_path_with_parent_dirs(path: &str) -> PathBuf {
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

fn show_home_interface(prompt: &str) {
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

fn select_no_projects_found() {
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

fn quit<T>() -> T {
    println!("Goodbye!");
    std::process::exit(0);
}

trait IntoString {
    fn into_string(self) -> Result<String, DynErr>;
}

impl IntoString for std::ffi::OsString {
    fn into_string(self) -> Result<String, DynErr> {
        self.into_string().map_err(|err| err.into())
    }
}

fn show_add_project_interface() {
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

enum Dialogue<'a> {
    Select(Select<'a>),
    MultiSelect(MultiSelect<'a>),
    // Confirm(Confirm<'a>),
    // Input(Input<'a, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Default)]
struct Project {
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

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.path)
    }
}

fn load_projects_from_disk() -> Vec<Project> {
    let mut file = open_projects_file(true, false, false);
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let projects_set: HashSet<Project> = serde_json::from_str(&json).unwrap_or_default();
    let mut projects: Vec<Project> = projects_set.into_iter().collect();
    // sort by last opened (most recent first)
    projects.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    projects
}

fn load_projects() -> Vec<Project> {
    let projects = PROJECTS.lock().unwrap();
    projects.to_vec()
}

fn save_projects(projects: &[Project]) {
    let mut file = File::create(get_config_dir().join("projects.json")).unwrap();
    let json = serde_json::to_string_pretty(&projects).unwrap();
    file.write_all(json.as_bytes()).unwrap();
    *PROJECTS.lock().unwrap() = projects.to_vec();
}

fn add_project(name: &str, path: &str) {
    let mut projects = load_projects();
    let default_path = env::current_dir().unwrap();
    let default_name = default_path.file_name().unwrap().to_str().unwrap();
    let name = if name.is_empty() { default_name } else { name };
    let path = if path.is_empty() {
        PathBuf::from(default_path.to_str().unwrap())
    } else {
        PathBuf::from(path).canonicalize().unwrap()
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

fn show_overwrite_project_interface(project: &Project) {
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

fn project_already_exists(name_or_path: &str) -> bool {
    let projects = load_projects();
    projects
        .iter()
        .any(|p| p.name == name_or_path || p.path == name_or_path)
}

fn show_select_projects_interface(action: Action, prompt: Option<&str>) {
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

fn delete_project(name: &str) {
    let mut projects = load_projects();
    projects.retain(|project| project.name != name);
    save_projects(&projects);
}

fn delete_projects(names: &[&str], also_delete_dir: bool) {
    let mut projects = load_projects();
    if also_delete_dir {
        for name in names {
            let project = projects
                .iter()
                .find(|project| project.name == *name)
                .unwrap();
            std::fs::remove_dir_all(&project.path).unwrap_or_else(|_| {
                println!("Failed to delete project directory: {}", project.path)
            });
        }
    }
    projects.retain(|project| !names.contains(&project.name.as_str()));
    save_projects(&projects);
}

/// Shows an interface for editing a project and saves the changes.
fn edit_project(name: &str) {
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
enum OpenAction {
    /// Open the project in the terminal (cd into the project folder)
    OpenInTerminal,
    /// Open the project in the default editor
    OpenInEditor,
}

fn open_project(name: &str, open_action: OpenAction, replace_editor: bool) {
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

fn change_directory(new_dir: &str) -> io::Result<()> {
    let path = std::path::Path::new(&new_dir);
    if path.exists() && path.is_dir() {
        env::set_current_dir(path)?;
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Command::new(shell).status()?;
    } else {
        eprintln!("cd: {}: No such file or directory", new_dir);
    }

    Ok(())
}

fn open_in_editor(path: &str, replace_editor: bool) -> io::Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(&editor)
        .arg(path)
        .arg(if replace_editor && editor == "code" { "--reuse-window" } else { "" })
        .status()?;
    Ok(())
}

fn get_config_dir() -> std::path::PathBuf {
    // check if a .config folder exists in the home directory
    let home_dir = dirs::home_dir().unwrap();
    let xdg_config_dir = home_dir.join(".config");
    let base_dir = if xdg_config_dir.exists() {
        xdg_config_dir
    } else {
        // use the home directory
        home_dir
    };
    let config_dir = base_dir.join("tpm");
    if !config_dir.exists() {
        std::fs::create_dir(&config_dir).unwrap();
    }

    config_dir
}

fn open_projects_file(read: bool, write: bool, create: bool) -> File {
    let config_dir = get_config_dir();
    let projects_file = config_dir.join("projects.json");
    // if the file doesn't exist, create it
    if !projects_file.exists() {
        File::create(&projects_file).unwrap();
    }
    std::fs::OpenOptions::new()
        .read(read)
        .write(write)
        .create(create)
        .open(projects_file)
        .unwrap()
}
