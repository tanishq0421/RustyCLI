use crate::command::{Command, Operator};
use crate::env_vars::Environment;
use crate::job_control::JobControl;
use nix::sys::wait::{waitpid, WaitPidFlag};
use nix::unistd::{dup2, execvpe, fork, pipe, ForkResult, Pid};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::prelude::RawFd;
use std::process::exit;

pub fn execute_commands(
    mut commands: Vec<Command>,
    env: &Environment,
    jobs: &mut JobControl,
) -> io::Result<()> {
    if commands.len() == 1 {
        execute_single_command(&mut commands[0], env, jobs)
    } else {
        execute_pipeline(commands, env, jobs)
    }
}

fn execute_single_command(
    command: &mut Command,
    env: &Environment,
    jobs: &mut JobControl,
) -> io::Result<()> {
    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            handle_redirections(command)?;

            let cmd_cstring = CString::new(command.name.clone()).unwrap();
            let args_cstring: Vec<CString> = command
                .args
                .iter()
                .map(|arg| CString::new(arg.as_str()).unwrap())
                .collect();

            let mut argv: Vec<&CString> = Vec::with_capacity(args_cstring.len() + 1);
            argv.push(&cmd_cstring);
            for arg in &args_cstring {
                argv.push(arg);
            }

            let envp_cstring: Vec<CString> = env
                .vars
                .iter()
                .map(|(k, v)| {
                    let mut kv = k.clone();
                    kv.push('=');
                    kv.push_str(v);
                    CString::new(kv).unwrap()
                })
                .collect();

            let envp: Vec<&CString> = envp_cstring.iter().collect();

            if let Err(e) = execvpe(&cmd_cstring, &argv, &envp) {
                eprintln!("Failed to execute {}: {}", command.name, e);
                exit(1);
            }
            Ok(())
        }
        Ok(ForkResult::Parent { child }) => {
            if command.background {
                let job_id = jobs.add_job(child);
                println!("[{}] {}", job_id, child);
            } else {
                waitpid(child, None).expect("Failed to wait on child");
            }
            Ok(())
        }
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
}

fn execute_pipeline(
    commands: Vec<Command>,
    env: &Environment,
    jobs: &mut JobControl,
) -> io::Result<()> {
    let mut pids = Vec::new();
    let mut fds = Vec::new();

    for i in 0..commands.len() {
        let (stdin_fd, stdout_fd) = if i == 0 {
            (None, Some(pipe()?))
        } else if i == commands.len() - 1 {
            (Some(fds.pop().unwrap()), None)
        } else {
            let pipefds = pipe()?;
            (Some(fds.pop().unwrap()), Some(pipefds))
        };

        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                if let Some((read_fd, _)) = stdin_fd {
                    dup2(read_fd, 0).expect("dup2 failed");
                }
                if let Some((_, write_fd)) = stdout_fd {
                    dup2(write_fd, 1).expect("dup2 failed");
                }

                // Close all pipe file descriptors
                for &(read_fd, write_fd) in &fds {
                    nix::unistd::close(read_fd).unwrap();
                    nix::unistd::close(write_fd).unwrap();
                }

                handle_redirections(&commands[i])?;

                let cmd_cstring = CString::new(commands[i].name.clone()).unwrap();
                let args_cstring: Vec<CString> = commands[i]
                    .args
                    .iter()
                    .map(|arg| CString::new(arg.as_str()).unwrap())
                    .collect();

                let mut argv: Vec<&CString> = Vec::with_capacity(args_cstring.len() + 1);
                argv.push(&cmd_cstring);
                for arg in &args_cstring {
                    argv.push(arg);
                }

                let envp_cstring: Vec<CString> = env
                    .vars
                    .iter()
                    .map(|(k, v)| {
                        let mut kv = k.clone();
                        kv.push('=');
                        kv.push_str(v);
                        CString::new(kv).unwrap()
                    })
                    .collect();

                let envp: Vec<&CString> = envp_cstring.iter().collect();

                if let Err(e) = execvpe(&cmd_cstring, &argv, &envp) {
                    eprintln!("Failed to execute {}: {}", commands[i].name, e);
                    exit(1);
                }
            }
            Ok(ForkResult::Parent { child }) => {
                if let Some((read_fd, write_fd)) = stdin_fd {
                    nix::unistd::close(read_fd).unwrap();
                    nix::unistd::close(write_fd).unwrap();
                }
                if let Some((read_fd, write_fd)) = stdout_fd {
                    fds.push((read_fd, write_fd));
                }
                pids.push(child);
            }
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }

    // Wait for all child processes
    for pid in pids {
        waitpid(pid, None).expect("Failed to wait on child");
    }

    Ok(())
}

fn handle_redirections(command: &Command) -> io::Result<()> {
    if let Some(ref input_file) = command.input_redirection {
        let file = File::open(input_file)?;
        dup2(file.as_raw_fd(), 0).expect("dup2 failed");
    }

    if let Some(ref output_file) = command.output_redirection {
        let file = if command.append_output {
            OpenOptions::new().create(true).append(true).open(output_file)?
        } else {
            OpenOptions::new().create(true).write(true).truncate(true).open(output_file)?
        };
        dup2(file.as_raw_fd(), 1).expect("dup2 failed");
    }

    Ok(())
}
