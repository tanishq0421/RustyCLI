use crate::env_vars::Environment;
use owo_colors::OwoColorize;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct PromptInfo {
    pub cwd: String,
    pub branch: Option<String>,
    pub dirty: bool,
}

/// Build a `PromptInfo` from the current process state.
pub fn current(env: &Environment) -> PromptInfo {
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("?"));
    let cwd = abbreviate_home(&cwd_path, env);
    let (branch, dirty) = git_info().unwrap_or((None, false));
    PromptInfo { cwd, branch, dirty }
}

fn abbreviate_home(path: &std::path::Path, env: &Environment) -> String {
    if let Some(home) = env.get_var("HOME") {
        if let Ok(stripped) = path.strip_prefix(home) {
            if stripped.as_os_str().is_empty() {
                return "~".into();
            }
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

fn git_info() -> Option<(Option<String>, bool)> {
    let repo = git2::Repository::discover(".").ok()?;
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(false);
    let dirty = repo
        .statuses(Some(&mut opts))
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    Some((branch, dirty))
}

/// Render the prompt as an ANSI-colored string.
pub fn render(info: &PromptInfo) -> String {
    let mut out = format!("{}", info.cwd.cyan().bold());
    if let Some(b) = &info.branch {
        out.push(' ');
        out.push_str(&format!("{}", b.green()));
        if info.dirty {
            out.push_str(&format!("{}", "*".red()));
        }
    }
    out.push(' ');
    out.push_str(&format!("{} ", "❯".bright_magenta()));
    out
}
