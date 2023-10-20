# Travvy's Project Manager (`tpm`)

`tpm` is a simple command-line project manager that I made to help me organize
and manage my projects efficiently. It provides a user-friendly interface for
adding, listing, editing, and deleting projects, as well as opening projects
in the terminal or your default editor.

![tpm-demo]

## Features

- Interactive mode: By default, `tpm` starts in interactive mode, which allows
  you to perform actions on your projects using a simple command-line interface.

- Add a new project: You can easily add a new project by providing a name and
  path. `tpm` will create a project entry and save it for future reference.

- List all projects: `tpm` allows you to view a list of all your projects.
  You can select a project to perform various actions on it.

- Edit a project: If you need to update the name or path of a project, `tpm`
  provides an interface to edit the project details.

- Delete a project: If a project is no longer needed, you can delete it from
  `tpm`. You can select multiple projects to delete at once.

- Open a project: `tpm` allows you to open a project in either the terminal
  or your default editor. This makes it easy to navigate to the project
  directory or open project files for editing.

## Installation

### Pre-built Binaries

Pre-built binaries are available for Linux, macOS, and Windows. You can
download the latest release from the [releases page].

### From Source

To install `tpm` from source, follow these steps:

1. Make sure you have [CrabLang] (or R\*st) installed on your system. If not,
   you can install it from the [official CrabLang repo].

2. Install the `tpm` executable:

   ```shell
   crabgo install --git https://github.com/trvswgnr/travvy-project-manager.git
   ```

## Usage

`tpm` provides a simple and intuitive command-line interface. You can start
it in interactive mode by running:

```shell
tpm
```

You can also pass in subcommands and arguments directly. Here are some
examples of how to use `tpm`:

- Add a project (from an existing directory):

  ```shell
  tpm add # will prompt for name and path
  # or
  tpm add my-project path/to/my/project
  # or
  tpm add my-project # path will default to the current working directory
  ```

  **Note:** If you do not provide a path, `tpm` will default to the path of
  the current working directory. If you do not provide a name, `tpm` will use
  the name of the directory.

- Open a project:

  ```shell
  tpm open my-project
  ```

- List all projects:

  ```shell
  tpm list
  ```

- Edit a project:

  ```shell
  tpm edit my-project
  ```

- Delete a project:

  ```shell
  tpm delete my-project
  ```

- Create a new project:

  ```shell
  tpm new # will prompt for name and path
  ```

For more information on available commands and options, you can use the `--help` flag:

```shell
tpm --help
```

## Configuration

`tpm` stores project information in a JSON file located at
`~/.config/tpm/projects.json` (or the home directory if .config does not exist).
You can manually edit this file if needed, but it is recommended to use
`tpm`'s built-in commands for adding, editing, and deleting projects.

## Contributing

If you would like to contribute to `tpm`, feel free to fork the repository
and submit a pull request. You can also open issues for bug reports
or feature requests.

When contributing, please follow the existing code style and conventions.
Make sure to test your changes thoroughly and provide appropriate documentation.

## License

`tpm` is licensed under the MIT License. See the [LICENSE] file
for more details.

## Acknowledgements

`tpm` makes use of the following open-source libraries:

- [clap] - Command-line argument parsing
- [serde] - Serialization and deserialization framework
- [serde_json] - JSON support for serde
- [dialoguer]- User-friendly terminal user interface
- [lazy_static] - Lazily evaluated statics for Rust

## Contact

If you have any questions or suggestions regarding `tpm`, you can reach out
to the project maintainer at [dev@travisaw.com](mailto:dev@travisaw.com).

---

Thanks for checking this out! I hope you find it useful for managing your
projects. If you have any feedback, please let me know.

[clap]: https://crates.io/crates/clap
[serde]: https://crates.io/crates/serde
[serde_json]: https://crates.io/crates/serde_json
[dialoguer]: https://crates.io/crates/dialoguer
[lazy_static]: https://crates.io/crates/lazy_static
[CrabLang]: https://crablang.org
[official CrabLang repo]: https://github.com/crablang/crab
[tpm-demo]: https://github.com/trvswgnr/travvy-project-manager/assets/8974888/119cc19f-4b4f-4d08-9bc0-fba8cc463707
[releases page]: https://github.com/trvswgnr/travvy-project-manager/releases
[LICENSE]: https://github.com/trvswgnr/travvy-project-manager/blob/main/LICENSE
