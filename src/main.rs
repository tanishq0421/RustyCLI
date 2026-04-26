mod builtins;
mod command;
mod env_vars;
mod executor;
mod job_control;
mod parser;
mod signal_handler;

use anyhow::Result;
use builtins::{handle_builtin, ShellEditor};
use env_vars::Environment;
use executor::{execute_pipeline, redirect_guard};
use job_control::JobControl;
use parser::{expand_variables, parse_pipelines};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use signal_handler::{setup_signal_handlers, take_interrupt};

fn main() -> Result<()> {
    let mut env = Environment::new();
    let mut rl: ShellEditor = Editor::new()?;
    let mut jobs = JobControl::new();

    setup_signal_handlers();

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if take_interrupt() { /* clear flag */ }

                let expanded = expand_variables(line.trim(), &env);
                let pipelines = match parse_pipelines(&expanded) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("rustycli: parse error: {}", e);
                        continue;
                    }
                };

                for pipeline in &pipelines {
                    if pipeline.stages.is_empty() {
                        continue;
                    }
                    // Single-stage pipelines may be a builtin (run in-process).
                    // Builtins inside pipes (stages > 1) fall through to fork+exec
                    // which means non-trivial builtins like `cd` won't work mid-pipe;
                    // that's acceptable POSIX-ish behavior.
                    if pipeline.stages.len() == 1 {
                        let cmd = &pipeline.stages[0];
                        if builtins::BUILTINS.contains(&cmd.name.as_str()) {
                            let needs_redir =
                                cmd.input_redirection.is_some() || cmd.output_redirection.is_some();
                            if needs_redir {
                                match redirect_guard(cmd) {
                                    Ok(_g) => {
                                        let _ = handle_builtin(cmd, &mut env, &mut rl, &mut jobs);
                                        // _g drops here, restoring stdio.
                                    }
                                    Err(e) => eprintln!("rustycli: {}: {:#}", cmd.name, e),
                                }
                            } else {
                                let _ = handle_builtin(cmd, &mut env, &mut rl, &mut jobs);
                            }
                            continue;
                        }
                    }
                    if let Err(e) = execute_pipeline(pipeline, &env, &mut jobs) {
                        eprintln!("rustycli: {:#}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C at the prompt: just print a fresh line and continue.
                let _ = take_interrupt();
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D: exit cleanly.
                break;
            }
            Err(e) => {
                eprintln!("rustycli: readline error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
