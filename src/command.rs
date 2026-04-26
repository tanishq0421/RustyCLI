use std::fmt;

#[derive(Debug, Clone)]
pub struct Redirect {
    pub path: String,
    pub append: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    /// Parallel to `args`. `true` means the argument was quoted in the source
    /// (so glob/var expansion may want to leave it alone).
    pub args_quoted: Vec<bool>,
    pub input_redirection: Option<String>,
    pub output_redirection: Option<Redirect>,
}

impl Command {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Pipeline {
    pub stages: Vec<Command>,
    pub background: bool,
}

impl Pipeline {
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty() || (self.stages.len() == 1 && self.stages[0].is_empty())
    }
}

impl fmt::Display for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self
            .stages
            .iter()
            .map(|c| {
                let mut s = c.name.clone();
                for a in &c.args {
                    s.push(' ');
                    s.push_str(a);
                }
                s
            })
            .collect();
        write!(
            f,
            "{}{}",
            parts.join(" | "),
            if self.background { " &" } else { "" }
        )
    }
}
