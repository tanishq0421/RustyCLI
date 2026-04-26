use std::collections::HashMap;
use std::env;

/// Mirror of the process environment. Mutations are also written through to
/// `std::env` so that subprocesses spawned via `fork`/`execvp` inherit them.
pub struct Environment {
    pub vars: HashMap<String, String>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: env::vars().collect(),
        }
    }

    /// Construct an empty environment (used by tests).
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn set_var(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
        env::set_var(key, value);
    }

    pub fn unset_var(&mut self, key: &str) {
        self.vars.remove(key);
        env::remove_var(key);
    }

    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
