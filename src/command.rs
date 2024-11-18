#[derive(Debug, Clone)]
pub enum Operator {
    Pipe,
    RedirectIn,
    RedirectOut,
    AppendOut,
    Background,
    None,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub operator: Operator,
    pub next: Option<Box<Command>>,
    pub input_redirection: Option<String>,
    pub output_redirection: Option<String>,
    pub append_output: bool,
    pub background: bool,
}

impl Command {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
}
