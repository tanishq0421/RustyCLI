/// Expand a single argument using shell-style globbing.
///
/// - If `arg` contains no glob metacharacters, returns `[arg]`.
/// - If glob expansion succeeds and matches at least one path, returns those paths.
/// - If nothing matches (or the pattern is invalid), returns `[arg]` literally
///   — matching bash's default `nullglob=off` behavior.
pub fn expand(arg: &str) -> Vec<String> {
    if !has_glob_meta(arg) {
        return vec![arg.to_string()];
    }
    match glob::glob(arg) {
        Ok(paths) => {
            let v: Vec<String> = paths
                .flatten()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            if v.is_empty() {
                vec![arg.to_string()]
            } else {
                v
            }
        }
        Err(_) => vec![arg.to_string()],
    }
}

fn has_glob_meta(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_meta_passes_through() {
        assert_eq!(expand("plain.txt"), vec!["plain.txt"]);
    }

    #[test]
    fn unmatched_pattern_returns_literal() {
        assert_eq!(expand("nope_xxx_*.zzz"), vec!["nope_xxx_*.zzz"]);
    }

    #[test]
    fn matched_pattern_expands() {
        // src/*.rs should match at least lib.rs / main.rs at the project root.
        let v = expand("src/*.rs");
        assert!(v.len() > 1, "expected multiple .rs matches, got {:?}", v);
        assert!(v.iter().any(|p| p.ends_with("parser.rs")));
    }
}
