use crate::command::Command;
use crate::env_vars::Environment;
use crate::job_control::JobControl;
use rustyline::Editor;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

pub fn handle_builtin(
    command: &Command,
    env: &mut Environment,
    rl: &mut Editor<()>,
    jobs: &mut JobControl,
) -> bool {
    match command.name.as_str() {
        "cd" => {
            let dir = command.args.get(0).map_or("/", |s| s);
            if let Err(e) = env::set_current_dir(dir) {
                eprintln!("cd: {}", e);
            }
            true
        }
        "pwd" => {
            if let Ok(path) = env::current_dir() {
                println!("{}", path.display());
            } else {
                eprintln!("pwd: failed to get current directory");
            }
            true
        }
        "echo" => {
            let output = command.args.join(" ");
            println!("{}", output);
            true
        }
        "export" => {
            for arg in &command.args {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() == 2 {
                    env.set_var(parts[0], parts[1]);
                } else {
                    eprintln!("export: invalid format");
                }
            }
            true
        }
        "unset" => {
            for var in &command.args {
                env.vars.remove(var);
            }
            true
        }
        "history" => {
            for (idx, entry) in rl.history().iter().enumerate() {
                println!("  {}  {}", idx + 1, entry);
            }
            true
        }
        "clear" => {
            // Clears the terminal screen
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().unwrap();
            true
        }
        "exit" => {
            process::exit(0);
        }
        "jobs" => {
            jobs.list_jobs();
            true
        }
        "fg" => {
            if let Some(job_id_str) = command.args.get(0) {
                if let Ok(job_id) = job_id_str.parse::<u32>() {
                    if let Some(pid) = jobs.get_job(job_id) {
                        jobs.bring_job_to_foreground(job_id, pid);
                    } else {
                        eprintln!("fg: job {} not found", job_id);
                    }
                } else {
                    eprintln!("fg: invalid job ID");
                }
            } else {
                eprintln!("fg: missing job ID");
            }
            true
        }
        "help" => {
            println!("Available built-in commands:");
            println!("cd, pwd, echo, export, unset, history, clear, exit, jobs, fg, help");
            true
        }
        _ => false, // Not a built-in command
    }
}
