use crate::alias::AliasTable;
use crate::env_vars::Environment;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Path to the rcfile: `$HOME/.rustyrc` if HOME is set.
pub fn rc_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".rustyrc"))
}

/// Read `~/.rustyrc` if present and apply `alias` and `export` directives.
/// Lines starting with `#` and blank lines are ignored.
pub fn load(aliases: &mut AliasTable, env: &mut Environment) -> Result<()> {
    let Some(path) = rc_path() else { return Ok(()) };
    if !path.exists() {
        return Ok(());
    }
    let body = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    for (lineno, raw) in body.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("alias ") {
            match parse_alias_line(rest) {
                Some((name, value)) => aliases.set(name, value),
                None => eprintln!(
                    "rustycli: {}:{}: invalid alias syntax",
                    path.display(),
                    lineno + 1
                ),
            }
        } else if let Some(rest) = line.strip_prefix("export ") {
            if let Some((k, v)) = rest.split_once('=') {
                let v = strip_quotes(v.trim());
                env.set_var(k.trim(), v);
            }
        }
    }
    Ok(())
}

/// Parse `name='value'` or `name="value"` or `name=value`.
fn parse_alias_line(s: &str) -> Option<(&str, &str)> {
    let (name, value) = s.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let value = strip_quotes_str(value.trim());
    Some((name, value))
}

fn strip_quotes(s: &str) -> &str {
    strip_quotes_str(s)
}

fn strip_quotes_str(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_double_quoted_alias() {
        assert_eq!(parse_alias_line("ll=\"ls -la\""), Some(("ll", "ls -la")));
    }

    #[test]
    fn parses_single_quoted_alias() {
        assert_eq!(
            parse_alias_line("gs='git status'"),
            Some(("gs", "git status"))
        );
    }

    #[test]
    fn parses_unquoted_alias() {
        assert_eq!(parse_alias_line("g=git"), Some(("g", "git")));
    }

    #[test]
    fn rejects_empty_name() {
        assert_eq!(parse_alias_line("=oops"), None);
    }
}
