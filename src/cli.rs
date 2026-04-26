use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rustycli", version, about = "A toy POSIX-ish shell in Rust")]
pub struct Cli {
    /// Execute a single command string and exit.
    #[arg(short = 'c', long, value_name = "CMD")]
    pub command: Option<String>,

    /// Script file to run (positional). Reads commands one per line.
    pub script: Option<PathBuf>,

    /// Suppress reading ~/.rustyrc on startup.
    #[arg(long)]
    pub norc: bool,
}
