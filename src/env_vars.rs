use std::collections::HashMap;
use std::env;

pub struct Environment {
    pub vars: HashMap<String, String>,
}

impl Environment {
    /// Initializes the environment with current environment variables
    pub fn new() -> Self {
        Self {
            vars: env::vars().collect(),
        }
    }

    /// Sets an environment variable
    pub fn set_var(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }

    /// Gets an environment variable
    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
}
