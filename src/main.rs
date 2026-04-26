use anyhow::Result;
use clap::Parser;
use rustycli::{run_interactive, run_reader, run_string, Cli, ShellState};
use std::fs::File;
use std::io::BufReader;
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let mut state = ShellState::new(!cli.norc);

    let code = match (cli.command.as_deref(), cli.script.as_deref()) {
        (Some(cmd), _) => run_string(cmd, &mut state)?,
        (None, Some(path)) => {
            let f = File::open(path)?;
            run_reader(BufReader::new(f), &mut state)?
        }
        (None, None) => run_interactive(&mut state)?,
    };
    // Clamp shell exit codes into 0–255.
    Ok(ExitCode::from((code & 0xff) as u8))
}
