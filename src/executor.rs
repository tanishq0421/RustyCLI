use crate::command::{Command, Pipeline};
use crate::env_vars::Environment;
use crate::job_control::JobControl;
use anyhow::{anyhow, Context, Result};
use nix::errno::Errno;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup, dup2, execvp, fork, pipe, ForkResult};
use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::exit;

pub fn execute_pipeline(
    pipeline: &Pipeline,
    _env: &Environment,
    jobs: &mut JobControl,
) -> Result<i32> {
    if pipeline.stages.is_empty() {
        return Ok(0);
    }

    let n = pipeline.stages.len();

    // Create n-1 pipes up front so each adjacent pair shares one.
    let mut pipes: Vec<(RawFd, RawFd)> = Vec::with_capacity(n.saturating_sub(1));
    for _ in 0..n.saturating_sub(1) {
        let (r, w) = pipe().context("failed to create pipe")?;
        pipes.push((r, w));
    }

    let mut pids = Vec::with_capacity(n);

    for (i, cmd) in pipeline.stages.iter().enumerate() {
        let stdin_fd = if i > 0 { Some(pipes[i - 1].0) } else { None };
        let stdout_fd = if i + 1 < n { Some(pipes[i].1) } else { None };

        // SAFETY: single-threaded shell; fork's safety bar is not violating async-signal rules.
        match unsafe { fork() }.context("fork failed")? {
            ForkResult::Child => {
                // Wire stdin/stdout for this stage.
                if let Some(fd) = stdin_fd {
                    dup2(fd, 0).expect("dup2 stdin");
                }
                if let Some(fd) = stdout_fd {
                    dup2(fd, 1).expect("dup2 stdout");
                }
                // Close all pipe fds the child no longer needs.
                for (r, w) in &pipes {
                    let _ = close(*r);
                    let _ = close(*w);
                }

                if let Err(e) = apply_redirections(cmd) {
                    eprintln!("rustycli: {}: {}", cmd.name, e);
                    exit(1);
                }

                let cmd_cstring = match CString::new(cmd.name.as_str()) {
                    Ok(c) => c,
                    Err(_) => {
                        eprintln!("rustycli: command name contains NUL byte");
                        exit(126);
                    }
                };
                let mut owned: Vec<CString> = Vec::with_capacity(cmd.args.len() + 1);
                owned.push(cmd_cstring.clone());
                for a in &cmd.args {
                    match CString::new(a.as_str()) {
                        Ok(c) => owned.push(c),
                        Err(_) => {
                            eprintln!("rustycli: argument contains NUL byte");
                            exit(126);
                        }
                    }
                }
                let argv: Vec<&CString> = owned.iter().collect();

                match execvp(&cmd_cstring, &argv) {
                    Ok(_) => unreachable!("execvp returns only on error"),
                    Err(Errno::ENOENT) => {
                        eprintln!("rustycli: {}: command not found", cmd.name);
                        exit(127);
                    }
                    Err(e) => {
                        eprintln!("rustycli: {}: {}", cmd.name, e);
                        exit(126);
                    }
                }
            }
            ForkResult::Parent { child } => {
                pids.push(child);
            }
        }
    }

    // Parent must close every pipe fd so EOF propagates.
    for (r, w) in &pipes {
        let _ = close(*r);
        let _ = close(*w);
    }

    if pipeline.background {
        if let Some(&pid) = pids.first() {
            let job_id = jobs.add_job(pid);
            println!("[{}] {}", job_id, pid);
        }
        return Ok(0);
    }

    let mut last_status = 0;
    for pid in pids {
        match waitpid(pid, None).context("waitpid failed")? {
            WaitStatus::Exited(_, code) => last_status = code,
            WaitStatus::Signaled(_, sig, _) => last_status = 128 + sig as i32,
            _ => {}
        }
    }
    Ok(last_status)
}

/// Save current stdio fds, apply `command`'s redirections to fds 0/1, and
/// return a guard that restores the originals when dropped. Used to wrap
/// builtins (which run in the shell process) so that `echo hi > out` works.
pub fn redirect_guard(command: &Command) -> Result<RedirectGuard> {
    use std::io::Write;
    // Flush user-space buffers before swapping the underlying fd.
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();

    let mut guard = RedirectGuard {
        saved_stdin: None,
        saved_stdout: None,
    };
    if command.input_redirection.is_some() {
        guard.saved_stdin = Some(dup(0).context("dup stdin")?);
    }
    if command.output_redirection.is_some() {
        guard.saved_stdout = Some(dup(1).context("dup stdout")?);
    }
    if let Err(e) = apply_redirections(command) {
        // Restore immediately on failure so we don't leave fds wedged.
        drop(guard);
        return Err(e);
    }
    Ok(guard)
}

pub struct RedirectGuard {
    saved_stdin: Option<RawFd>,
    saved_stdout: Option<RawFd>,
}

impl Drop for RedirectGuard {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        if let Some(fd) = self.saved_stdin.take() {
            let _ = dup2(fd, 0);
            let _ = close(fd);
        }
        if let Some(fd) = self.saved_stdout.take() {
            let _ = dup2(fd, 1);
            let _ = close(fd);
        }
    }
}

fn apply_redirections(command: &Command) -> Result<()> {
    if let Some(input_file) = &command.input_redirection {
        let file = File::open(input_file)
            .with_context(|| format!("cannot open {} for reading", input_file))?;
        dup2(file.as_raw_fd(), 0).map_err(|e| anyhow!("dup2 stdin: {}", e))?;
    }
    if let Some(redir) = &command.output_redirection {
        let file = if redir.append {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&redir.path)
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&redir.path)
        }
        .with_context(|| format!("cannot open {} for writing", redir.path))?;
        dup2(file.as_raw_fd(), 1).map_err(|e| anyhow!("dup2 stdout: {}", e))?;
    }
    Ok(())
}
