use crate::alias::AliasTable;
use crate::builtins::{self, handle_builtin, ShellEditor};
use crate::completion::RustyHelper;
use crate::env_vars::Environment;
use crate::executor::{execute_pipeline, redirect_guard};
use crate::job_control::JobControl;
use crate::parser::{expand_variables, parse_pipelines};
use crate::prompt;
use crate::rcfile;
use crate::signal_handler::{setup_signal_handlers, take_interrupt};
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io::{BufRead, Cursor};

pub struct ShellState {
    pub env: Environment,
    pub jobs: JobControl,
    pub aliases: AliasTable,
    pub last_status: i32,
}

impl ShellState {
    pub fn new(load_rc: bool) -> Self {
        let mut state = Self {
            env: Environment::new(),
            jobs: JobControl::new(),
            aliases: AliasTable::default(),
            last_status: 0,
        };
        if load_rc {
            if let Err(e) = rcfile::load(&mut state.aliases, &mut state.env) {
                eprintln!("rustycli: rc: {:#}", e);
            }
        }
        state
    }
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Execute one logical input line. Returns the last command's exit code.
pub fn dispatch(line: &str, state: &mut ShellState, rl: &mut ShellEditor) -> i32 {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return state.last_status;
    }
    let pipelines = match parse_pipelines(trimmed) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("rustycli: parse error: {}", e);
            return 2;
        }
    };
    let mut last = state.last_status;
    for pipeline in &pipelines {
        let mut pipeline = state.aliases.expand_pipeline(pipeline.clone());
        // Expand $VAR in name + args + redirection paths AFTER any earlier
        // pipeline in the same line has had a chance to mutate the env.
        for stage in &mut pipeline.stages {
            stage.name = expand_variables(&stage.name, &state.env);
            stage.args = stage
                .args
                .drain(..)
                .map(|a| expand_variables(&a, &state.env))
                .collect();
            if let Some(p) = &stage.input_redirection {
                stage.input_redirection = Some(expand_variables(p, &state.env));
            }
            if let Some(r) = &mut stage.output_redirection {
                r.path = expand_variables(&r.path, &state.env);
            }
        }
        // Glob expansion runs after variable expansion so `$DIR/*.rs` works.
        // Quoted args bypass globbing (bash semantics).
        for stage in &mut pipeline.stages {
            let mut new_args: Vec<String> = Vec::with_capacity(stage.args.len());
            let mut new_quoted: Vec<bool> = Vec::with_capacity(stage.args.len());
            for (arg, was_quoted) in stage.args.drain(..).zip(stage.args_quoted.drain(..)) {
                if was_quoted {
                    new_args.push(arg);
                    new_quoted.push(true);
                } else {
                    let expanded = crate::glob_expand::expand(&arg);
                    for e in expanded {
                        new_args.push(e);
                        new_quoted.push(false);
                    }
                }
            }
            stage.args = new_args;
            stage.args_quoted = new_quoted;
        }
        if pipeline.stages.is_empty() {
            continue;
        }
        if pipeline.stages.len() == 1 {
            let cmd = &pipeline.stages[0];
            if builtins::BUILTINS.contains(&cmd.name.as_str()) {
                let needs_redir =
                    cmd.input_redirection.is_some() || cmd.output_redirection.is_some();
                let code = if needs_redir {
                    match redirect_guard(cmd) {
                        Ok(_g) => {
                            handle_builtin(cmd, &mut state.env, rl, &mut state.jobs).unwrap_or(0)
                        }
                        Err(e) => {
                            eprintln!("rustycli: {}: {:#}", cmd.name, e);
                            1
                        }
                    }
                } else {
                    handle_builtin(cmd, &mut state.env, rl, &mut state.jobs).unwrap_or(0)
                };
                last = code;
                continue;
            }
        }
        match execute_pipeline(&pipeline, &state.env, &mut state.jobs) {
            Ok(c) => last = c,
            Err(e) => {
                eprintln!("rustycli: {:#}", e);
                last = 1;
            }
        }
    }
    state.last_status = last;
    last
}

/// Run a single command string (used by `-c "..."`).
pub fn run_string(cmd: &str, state: &mut ShellState) -> Result<i32> {
    let mut rl: ShellEditor = Editor::new()?;
    Ok(dispatch(cmd, state, &mut rl))
}

/// Run commands from any `BufRead` source (used by script mode and tests).
pub fn run_reader<R: BufRead>(reader: R, state: &mut ShellState) -> Result<i32> {
    let mut rl: ShellEditor = Editor::new()?;
    let mut last = 0;
    for line in reader.lines() {
        let line = line?;
        last = dispatch(&line, state, &mut rl);
    }
    Ok(last)
}

/// Convenience for tests — run a string through the script-mode pipeline.
pub fn run_script_string(script: &str, state: &mut ShellState) -> Result<i32> {
    run_reader(Cursor::new(script), state)
}

/// Interactive REPL with rustyline.
pub fn run_interactive(state: &mut ShellState) -> Result<i32> {
    setup_signal_handlers();

    let mut rl: ShellEditor = Editor::new()?;
    rl.set_helper(Some(RustyHelper::new()));

    loop {
        let prompt_str = prompt::render(&prompt::current(&state.env));
        match rl.readline(&prompt_str) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let _ = take_interrupt();
                dispatch(&line, state, &mut rl);
            }
            Err(ReadlineError::Interrupted) => {
                let _ = take_interrupt();
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("rustycli: readline error: {}", e);
                break;
            }
        }
    }
    Ok(state.last_status)
}
