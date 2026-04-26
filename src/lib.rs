//! RustyCLI — a small Unix-like shell in Rust.
//!
//! `main.rs` is a thin wrapper around the library entry points so that the
//! parser/executor can also be exercised by integration tests and fuzzers.

pub mod alias;
pub mod builtins;
pub mod cli;
pub mod command;
pub mod completion;
pub mod env_vars;
pub mod executor;
pub mod glob_expand;
pub mod job_control;
pub mod parser;
pub mod prompt;
pub mod rcfile;
pub mod runner;
pub mod signal_handler;

pub use cli::Cli;
pub use runner::{run_interactive, run_reader, run_string, ShellState};
