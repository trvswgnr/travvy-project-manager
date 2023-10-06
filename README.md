# Travvy's Project Manager (TPM)

TPM is a simple command-line project manager designed to help you organize and manage your projects efficiently. It provides a user-friendly interface for adding, listing, editing, and deleting projects, as well as opening projects in the terminal or default editor.

## Features

- Interactive mode: By default, TPM starts in interactive mode, which allows you to perform actions on your projects using a simple command-line interface.

- Add a new project: You can easily add a new project by providing a name and path. TPM will create a project entry and save it for future reference.

- List all projects: TPM allows you to view a list of all your projects. You can select a project to perform various actions on it.

- Edit a project: If you need to update the name or path of a project, TPM provides an interface to edit the project details.

- Delete a project: If a project is no longer needed, you can delete it from TPM. You can select multiple projects to delete at once.

- Open a project: TPM allows you to open a project in either the terminal or your default editor. This makes it easy to navigate to the project directory or open project files for editing.

## Installation

To install TPM, follow these steps:

1. Make sure you have [CrabLang](https://crablang.org) installed on your system. If not, you can install it from the [official CrabLang repo](https://github.com/crablang/crablang).

2. Clone the TPM repository to your local machine:

   ```shell
   git clone https://github.com/trvswgnr/travvy-project-manager.git
   ```

3. Navigate to the project directory:

   ```shell
   cd travvy-project-manager
   ```

4. Build the TPM executable:

   ```shell
   cargo install --path .
   ```

## Usage

TPM provides a simple and intuitive command-line interface. You can start it in interactive mode by running:

```shell
tpm
```

You can also pass in subcommands and arguments directly. Here are some examples of how to use TPM:

- Add a new project:

  ```shell
  tpm add -n "My Project" -p "/path/to/my/project"
  ```

- List all projects:

  ```shell
  tpm list
  ```

- Edit a project:

  ```shell
  tpm edit -n "My Project"
  ```

- Delete a project:

  ```shell
  tpm delete -n "My Project"
  ```

- Open a project:

  ```shell
  tpm open -n "My Project"
  ```

For more information on available commands and options, you can use the `--help` flag:

```shell
tpm --help
```

## Configuration

TPM stores project information in a JSON file located at `~/.config/tpm/projects.json`. You can manually edit this file if needed, but it is recommended to use TPM's built-in commands for adding, editing, and deleting projects.

## Contributing

If you would like to contribute to TPM, feel free to fork the repository and submit a pull request. You can also open issues for bug reports or feature requests.

When contributing, please follow the existing code style and conventions. Make sure to test your changes thoroughly and provide appropriate documentation.

## License

TPM is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Acknowledgements

TPM makes use of the following open-source libraries:

- [clap](https://crates.io/crates/clap) - Command-line argument parsing
- [serde](https://crates.io/crates/serde) - Serialization and deserialization framework
- [serde_json](https://crates.io/crates/serde_json) - JSON support for serde
- [dialoguer](https://crates.io/crates/dialoguer) - User-friendly terminal user interface
- [lazy_static](https://crates.io/crates/lazy_static) - Lazily evaluated statics for Rust

## Contact

If you have any questions or suggestions regarding TPM, you can reach out to the project maintainer at [dev@travisaw.com](mailto:dev@travisaw.com).

---

Thanks for checking this out! I hope you find it useful for managing your projects. If you have any feedback, please let me know.
