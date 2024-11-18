mod builtins;
mod command;
mod env_vars;
mod executor;
mod job_control;
mod parser;
mod signal_handler;

use builtins::handle_builtin;
use env_vars::Environment;
use executor::execute_commands;
use job_control::JobControl;
use parser::{expand_variables, parse_input};
use rustyline::Editor;
use signal_handler::setup_signal_handlers;

fn main() {
    let mut env = Environment::new();
    let mut rl = Editor::<()>::new();
    let mut jobs = JobControl::new();

    setup_signal_handlers();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let expanded_input = expand_variables(&line.trim(), &env);
                let commands = parse_input(&expanded_input);
                if commands.is_empty() {
                    continue;
                }

                // Handle built-in commands
                if handle_builtin(&commands[0], &mut env, &mut rl, &mut jobs) {
                    continue;
                }

                // Execute the commands
                if let Err(e) = execute_commands(commands, &env, &mut jobs) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(_) => {
                println!("Error reading line");
                break;
            }
        }
    }
}
