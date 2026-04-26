use crate::builtins::BUILTINS;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

pub struct RustyHelper {
    fname: FilenameCompleter,
}

impl RustyHelper {
    pub fn new() -> Self {
        Self {
            fname: FilenameCompleter::new(),
        }
    }
}

impl Default for RustyHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl Helper for RustyHelper {}
impl Hinter for RustyHelper {
    type Hint = String;
}
impl Highlighter for RustyHelper {}
impl Validator for RustyHelper {}

impl Completer for RustyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Slice the line up to the cursor, find the start of the current token.
        let head = &line[..pos];
        let token_start = head
            .rfind(|c: char| c.is_whitespace() || matches!(c, '|' | '<' | '>' | ';' | '&'))
            .map(|i| i + 1)
            .unwrap_or(0);
        let token = &head[token_start..];

        // $VAR completion — consume the leading '$'.
        if let Some(var_prefix) = token.strip_prefix('$') {
            let mut out = Vec::new();
            for (k, _) in std::env::vars() {
                if k.starts_with(var_prefix) {
                    out.push(Pair {
                        display: format!("${}", k),
                        replacement: format!("${}", k),
                    });
                }
            }
            return Ok((token_start, out));
        }

        // First word on the line → builtin / PATH binary completion.
        let leading_ws = head[..token_start]
            .chars()
            .all(|c| c.is_whitespace() || c == ';');
        if leading_ws {
            let out: Vec<Pair> = BUILTINS
                .iter()
                .filter(|b| b.starts_with(token))
                .map(|b| Pair {
                    display: (*b).to_string(),
                    replacement: (*b).to_string(),
                })
                .collect();
            // Defer to FilenameCompleter for things like `./script`.
            if token.contains('/') || token.starts_with('.') {
                if let Ok((start, more)) = self.fname.complete(line, pos, ctx) {
                    return Ok((start, more));
                }
            }
            return Ok((token_start, out));
        }

        // Otherwise: filesystem path completion.
        self.fname.complete(line, pos, ctx)
    }
}
