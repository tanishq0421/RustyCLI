# Rust Shell Project

A Rust-based command-line shell with Unix-like features, including advanced command parsing, process management, environment variable handling, piping, redirection, job control, and more. This shell leverages Rust's memory safety features and the nix crate to provide a reliable and robust experience

## Features

- **Command Parsing**: Supports advanced command parsing, including variable expansion, pipelines, and background execution.
- **Process Management**: Implements process forking and execution via the `nix` crate, enhancing reliability.
- **Built-in Commands**: Provides built-in shell commands like `cd`, `pwd`, `echo`, `export`, `unset`, `history`, `clear`, `jobs`, `fg`, and `help`.
- **Environment Variable Handling**: Allows setting, unsetting, and expanding environment variables within commands.
- **Piping and Redirection**: Supports piping between commands and input/output redirection using `|`, `>`, `>>`, and `<`.
- **Job Control**: Enables background execution of processes using `&` and provides job control commands.
- **Signal Handling**: Handles `SIGINT` (Ctrl+C) gracefully without exiting the shell abruptly.
- **Command History**: Integrates with `rustyline` to provide command history and line editing capabilities.
- **Modular Structure**: Organized into modules for scalability and maintainability.

---

## Project Structure
```bash
rustyCli/
├── Cargo.toml
└── src/
    ├── builtins.rs          # Built-in shell commands
    ├── command.rs           # Command structures
    ├── env_vars.rs          # Environment variable management
    ├── executor.rs          # Command execution logic
    ├── job_control.rs       # Job control for background processes
    ├── main.rs              # Entry point of the application
    ├── parser.rs            # Command parsing and variable expansion
    └── signal_handler.rs    # Signal handling (e.g., Ctrl+C)
```
---

## Installation

### Prerequisites

- **Rust**: Ensure you have Rust installed. You can install Rust using [rustup](https://www.rust-lang.org/tools/install).

### Clone the Repository

```bash
git clone https://github.com/tanishq0421/RustyCLI.git
cd src
```

### Build the Project
```bash
cargo build
```

### Running the Shell
```bash
cargo run
```

## Built-in Commands
The shell provides several built-in commands:

- **cd [DIR]:** Change the current directory to DIR. Defaults to / if DIR is not provided.
- **pwd:** Print the current working directory.
- **echo [ARGS]:** Display a line of text.
- **export VAR=VALUE:** Set an environment variable.
- **unset VAR:** Remove an environment variable.
- **history:** Display the command history.
- **clear:** Clear the terminal screen.
- **jobs:** List background jobs.
- **fg JOB_ID:** Bring a background job to the foreground.
- **help:** Display help information.
- **exit:** Exit the shell.


## Environment Variable Expansion
You can use environment variables in your commands by prefixing them with $.

```bash
> export MY_VAR=HelloWorld
> echo $MY_VAR
HelloWorld
```

## Command History
Use the history command to display the list of previously entered commands.

```bash
> ls
> pwd
> history
  1  ls
  2  pwd
  3  history
```

## Signal Handling
The shell handles SIGINT (Ctrl+C) gracefully. Pressing Ctrl+C will not exit the shell but will interrupt the current foreground process.

```bash
> sleep 30
# Press Ctrl+C
Received Ctrl+C. Type 'exit' to quit.
```


## Development

### Dependencies
The project uses the following crates:

- **nix:** For Unix-like system calls and signal handling.
- **rustyline:** For line editing and command history.
- **regex:** For parsing and environment variable expansion.

These dependencies are specified in Cargo.toml:

```bash
[dependencies]
nix = "0.26.2"
rustyline = "11.1.2"
regex = "1.7.1"
```
