use crate::command::Pipeline;
use std::collections::HashMap;

const MAX_ALIAS_DEPTH: usize = 16;

#[derive(Default, Debug, Clone)]
pub struct AliasTable {
    map: HashMap<String, String>,
}

impl AliasTable {
    pub fn set(&mut self, name: &str, value: &str) {
        self.map.insert(name.to_string(), value.to_string());
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.map.get(name).map(|s| s.as_str())
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(|s| s.as_str())
    }

    /// Recursively expand the *first token* of a command name. Cycles are
    /// detected by depth limit and by tracking already-seen names.
    pub fn expand_first_token(&self, tokens: &mut Vec<String>) {
        if tokens.is_empty() {
            return;
        }
        let mut seen = std::collections::HashSet::new();
        for _ in 0..MAX_ALIAS_DEPTH {
            let head = tokens[0].clone();
            if !seen.insert(head.clone()) {
                break; // cycle
            }
            let Some(value) = self.get(&head) else { break };
            // Tokenize alias value on whitespace (no shell quoting in rcfile values).
            let parts: Vec<String> = value.split_whitespace().map(String::from).collect();
            if parts.is_empty() {
                break;
            }
            // Replace head with parts, keep the rest of tokens.
            let rest = tokens.split_off(1);
            *tokens = parts;
            tokens.extend(rest);
        }
    }

    /// Apply alias expansion to every stage of a pipeline.
    pub fn expand_pipeline(&self, mut pipeline: Pipeline) -> Pipeline {
        for stage in &mut pipeline.stages {
            let original_arg_count = stage.args.len();
            let mut tokens = std::iter::once(stage.name.clone())
                .chain(stage.args.iter().cloned())
                .collect::<Vec<_>>();
            self.expand_first_token(&mut tokens);
            if let Some((first, rest)) = tokens.split_first() {
                stage.name = first.clone();
                stage.args = rest.to_vec();
                // Tokens introduced by alias expansion are unquoted; preserve
                // the original quoted-ness for any user-provided trailing args.
                let new_count = stage.args.len();
                let injected = new_count.saturating_sub(original_arg_count);
                let mut new_quoted = vec![false; injected];
                new_quoted.extend(stage.args_quoted.iter().copied());
                new_quoted.truncate(new_count);
                while new_quoted.len() < new_count {
                    new_quoted.push(false);
                }
                stage.args_quoted = new_quoted;
            }
        }
        pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expansion() {
        let mut t = AliasTable::default();
        t.set("ll", "ls -la");
        let mut tokens = vec!["ll".into(), "src".into()];
        t.expand_first_token(&mut tokens);
        assert_eq!(tokens, vec!["ls", "-la", "src"]);
    }

    #[test]
    fn no_match_is_noop() {
        let t = AliasTable::default();
        let mut tokens = vec!["echo".into(), "hi".into()];
        t.expand_first_token(&mut tokens);
        assert_eq!(tokens, vec!["echo", "hi"]);
    }

    #[test]
    fn recursive_expansion() {
        let mut t = AliasTable::default();
        t.set("a", "b 1");
        t.set("b", "echo");
        let mut tokens = vec!["a".into(), "x".into()];
        t.expand_first_token(&mut tokens);
        assert_eq!(tokens, vec!["echo", "1", "x"]);
    }

    #[test]
    fn cycle_does_not_loop_forever() {
        let mut t = AliasTable::default();
        t.set("a", "b");
        t.set("b", "a");
        let mut tokens = vec!["a".into()];
        t.expand_first_token(&mut tokens);
        // Stops as soon as we'd revisit "a"; final token is "b" or "a".
        assert!(tokens == vec!["a"] || tokens == vec!["b"]);
    }
}
