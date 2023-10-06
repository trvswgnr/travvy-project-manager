use clap::{App, Arg, SubCommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Write};
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
            .arg(Arg::with_name("name").short("n").takes_value(true))
            .arg(Arg::with_name("path").short("p").takes_value(true)),
    )
    .subcommand(SubCommand::with_name("list").about("List all projects"))
    .subcommand(
        SubCommand::with_name("delete")
            .about("Delete a project")
            .arg(Arg::with_name("name").short("n").takes_value(true)),
    )
    .subcommand(
        SubCommand::with_name("edit").about("Edit a project")
            .arg(Arg::with_name("name").short("n").takes_value(true)),
    )
    .subcommand(
        SubCommand::with_name("open")
            .about("Open a project")
            .arg(Arg::with_name("name").takes_value(true)),
    )
    .get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let name = add_matches.value_of("name").unwrap_or("");
            let path = add_matches.value_of("path").unwrap_or("");
            add_project(name, path);
        }
        ("list", Some(_)) => {
            show_select_projects_interface(Action::Open, None);
        }
        ("delete", Some(delete_matches)) => {
            let name = delete_matches.value_of("name").unwrap_or("");
            if name.is_empty() {
                show_select_projects_interface(Action::Delete, None);
                return;
            }
            delete_project(name);
        }
        ("edit", Some(edit_matches)) => {
            let name = edit_matches.value_of("name").unwrap_or("");
            edit_project(name);
        }
        ("open", Some(open_matches)) => {
            let name = open_matches.value_of("name").unwrap_or("");
            open_project(name, OpenAction::OpenInTerminal);
        }
        _ => show_home_interface("What would you like to do?"),
    }
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
        4 => quit(),
        _ => {}
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
    let current_dir = std::env::current_dir().unwrap();
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
}

fn add_project(name: &str, path: &str) {
    let mut projects = load_projects();
    let default_path = std::env::current_dir().unwrap();
    let default_name = default_path.file_name().unwrap().to_str().unwrap();
    let name = if name.is_empty() { default_name } else { name };
    let path = if path.is_empty() {
        default_path.to_str().unwrap()
    } else {
        path
    };
    let mut project = Project {
        name: name.to_string(),
        path: path.to_string(),
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

fn project_already_exists(name: &str) -> bool {
    let projects = load_projects();
    projects.iter().any(|p| p.name == name)
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
                    open_project(&project.name, OpenAction::OpenInTerminal);
                }
                1 => {
                    for project in selected_projects {
                        open_project(&project.name, OpenAction::OpenInEditor);
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
            for project in selected_projects {
                delete_project(&project.name);
            }
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

fn open_project(name: &str, open_action: OpenAction) {
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
                open_in_editor(&project.path).unwrap();
            }
        }
    } else {
        println!("Project not found");
    }
}

fn change_directory(new_dir: &str) -> io::Result<()> {
    let path = std::path::Path::new(&new_dir);
    if path.exists() && path.is_dir() {
        std::env::set_current_dir(path)?;
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Command::new(shell).status()?;
    } else {
        eprintln!("cd: {}: No such file or directory", new_dir);
    }

    Ok(())
}

fn open_in_editor(path: &str) -> io::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(editor).arg(path).status()?;
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
