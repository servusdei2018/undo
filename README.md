# undo

`undo` is a simple tool to track file modifications made by external programs and allows you to undo those changes later.

This allows you to safely edit files or run commands, with the ability to roll back changes if needed.

## Usage

```bash
Usage: undo [COMMAND]

Commands:
  clear   Clear the history of tracked file modifications
  list    List all modified files that can be reverted
  revert  Revert the changes made to a file (or all files)
  run     Run a command while tracking file modifications
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Installation

You can build `undo` from source using Cargo, the Rust package manager. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

1. Clone the repository:

    ```bash
    $ git clone https://github.com/servusdei2018/undo.git
    $ cd undo
    ```

2. Build the project:

    ```bash
    $ cargo build --release
    ```
    After this, the `undo` binary will be available in the target/release/ directory.

3. Optionally, move the binary to a directory in your $PATH to make it accessible globally:

    ```bash
    $ sudo mv target/release/undo /usr/local/bin/undo
    ```

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.
