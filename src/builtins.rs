use crate::command::Command;
use crate::completion::RustyHelper;
use crate::env_vars::Environment;
use crate::job_control::JobControl;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use std::env;
use std::io::{self, Write};
use std::process;

pub const BUILTINS: &[&str] = &[
    "cd", "pwd", "echo", "export", "unset", "history", "clear", "exit", "jobs", "fg", "help",
];

pub type ShellEditor = Editor<RustyHelper, DefaultHistory>;

/// Returns `Some(exit_code)` if `command` is a builtin (and was handled),
/// otherwise `None` so the executor can try to spawn it.
pub fn handle_builtin(
    command: &Command,
    env: &mut Environment,
    rl: &mut ShellEditor,
    jobs: &mut JobControl,
) -> Option<i32> {
    match command.name.as_str() {
        "cd" => {
            let dir = command
                .args
                .first()
                .cloned()
                .or_else(|| env.get_var("HOME").cloned())
                .unwrap_or_else(|| "/".to_string());
            if let Err(e) = env::set_current_dir(&dir) {
                eprintln!("cd: {}: {}", dir, e);
                return Some(1);
            }
            Some(0)
        }
        "pwd" => match env::current_dir() {
            Ok(path) => {
                println!("{}", path.display());
                Some(0)
            }
            Err(e) => {
                eprintln!("pwd: {}", e);
                Some(1)
            }
        },
        "echo" => {
            println!("{}", command.args.join(" "));
            Some(0)
        }
        "export" => {
            for arg in &command.args {
                match arg.split_once('=') {
                    Some((k, v)) if !k.is_empty() => env.set_var(k, v),
                    _ => {
                        eprintln!("export: invalid format: {}", arg);
                        return Some(1);
                    }
                }
            }
            Some(0)
        }
        "unset" => {
            for var in &command.args {
                env.unset_var(var);
            }
            Some(0)
        }
        "history" => {
            for (idx, entry) in rl.history().iter().enumerate() {
                println!("  {}  {}", idx + 1, entry);
            }
            Some(0)
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().ok();
            Some(0)
        }
        "exit" => {
            let code = command
                .args
                .first()
                .and_then(|a| a.parse::<i32>().ok())
                .unwrap_or(0);
            process::exit(code);
        }
        "jobs" => {
            jobs.list_jobs();
            Some(0)
        }
        "fg" => {
            let Some(job_id_str) = command.args.first() else {
                eprintln!("fg: missing job ID");
                return Some(1);
            };
            let Ok(job_id) = job_id_str.parse::<u32>() else {
                eprintln!("fg: invalid job ID");
                return Some(1);
            };
            let Some(pid) = jobs.get_job(job_id) else {
                eprintln!("fg: job {} not found", job_id);
                return Some(1);
            };
            jobs.bring_job_to_foreground(job_id, pid);
            Some(0)
        }
        "help" => {
            println!("Available built-in commands:");
            println!("  {}", BUILTINS.join(", "));
            Some(0)
        }
        _ => None,
    }
}
