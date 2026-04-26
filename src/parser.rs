use crate::command::{Command, Pipeline, Redirect};
use crate::env_vars::Environment;
use regex::Regex;
use std::iter::Peekable;
use std::vec::IntoIter;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("unterminated quoted string")]
    UnterminatedQuote,
    #[error("expected filename after redirection operator")]
    MissingRedirectionTarget,
    #[error("expected command after `|`")]
    EmptyPipelineStage,
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    /// Word(text, quoted)
    Word(String, bool),
    Pipe,
    RedirectIn,
    RedirectOut,
    AppendOut,
    Background,
    Semicolon,
}

pub fn expand_variables(input: &str, env: &Environment) -> String {
    let re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        env.get_var(&caps[1]).cloned().unwrap_or_default()
    })
    .into_owned()
}

pub fn parse_pipelines(input: &str) -> Result<Vec<Pipeline>, ParseError> {
    let tokens = tokenize(input)?;
    let mut pipelines = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while iter.peek().is_some() {
        if let Some(Token::Semicolon) = iter.peek() {
            iter.next();
            continue;
        }
        let pipeline = parse_pipeline(&mut iter)?;
        if !pipeline.is_empty() {
            pipelines.push(pipeline);
        }
    }
    Ok(pipelines)
}

fn parse_pipeline(iter: &mut Peekable<IntoIter<Token>>) -> Result<Pipeline, ParseError> {
    let mut pipeline = Pipeline::default();
    loop {
        let cmd = parse_command(iter)?;
        let cmd_was_empty = cmd.is_empty();
        if !cmd_was_empty {
            pipeline.stages.push(cmd);
        }
        match iter.peek() {
            Some(Token::Pipe) => {
                if cmd_was_empty {
                    return Err(ParseError::EmptyPipelineStage);
                }
                iter.next();
            }
            Some(Token::Background) => {
                iter.next();
                pipeline.background = true;
                break;
            }
            Some(Token::Semicolon) | None => break,
            _ => break,
        }
    }
    Ok(pipeline)
}

fn parse_command(iter: &mut Peekable<IntoIter<Token>>) -> Result<Command, ParseError> {
    let mut cmd = Command::default();
    while let Some(tok) = iter.peek() {
        match tok {
            Token::Word(_, _) => {
                if let Some(Token::Word(w, quoted)) = iter.next() {
                    if cmd.name.is_empty() {
                        cmd.name = w;
                    } else {
                        cmd.args.push(w);
                        cmd.args_quoted.push(quoted);
                    }
                }
            }
            Token::RedirectIn => {
                iter.next();
                match iter.next() {
                    Some(Token::Word(file, _)) => cmd.input_redirection = Some(file),
                    _ => return Err(ParseError::MissingRedirectionTarget),
                }
            }
            Token::RedirectOut => {
                iter.next();
                match iter.next() {
                    Some(Token::Word(file, _)) => {
                        cmd.output_redirection = Some(Redirect {
                            path: file,
                            append: false,
                        })
                    }
                    _ => return Err(ParseError::MissingRedirectionTarget),
                }
            }
            Token::AppendOut => {
                iter.next();
                match iter.next() {
                    Some(Token::Word(file, _)) => {
                        cmd.output_redirection = Some(Redirect {
                            path: file,
                            append: true,
                        })
                    }
                    _ => return Err(ParseError::MissingRedirectionTarget),
                }
            }
            Token::Pipe | Token::Background | Token::Semicolon => break,
        }
    }
    Ok(cmd)
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' | '\n' => {
                chars.next();
            }
            '|' => {
                chars.next();
                tokens.push(Token::Pipe);
            }
            '<' => {
                chars.next();
                tokens.push(Token::RedirectIn);
            }
            '&' => {
                chars.next();
                tokens.push(Token::Background);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::AppendOut);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }
            '"' | '\'' => {
                let quote = c;
                chars.next();
                let mut s = String::new();
                let mut closed = false;
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == quote {
                        closed = true;
                        break;
                    }
                    s.push(c);
                }
                if !closed {
                    return Err(ParseError::UnterminatedQuote);
                }
                tokens.push(Token::Word(s, true));
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if matches!(
                        c,
                        ' ' | '\t' | '\r' | '\n' | '|' | '<' | '>' | '&' | ';' | '"' | '\''
                    ) {
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                tokens.push(Token::Word(s, false));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env_vars::Environment;

    fn parse(input: &str) -> Vec<Pipeline> {
        parse_pipelines(input).unwrap()
    }

    #[test]
    fn parse_empty_input_returns_empty_vec() {
        assert!(parse("").is_empty());
        assert!(parse("   ").is_empty());
    }

    #[test]
    fn parse_single_command_no_args() {
        let p = parse("ls");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].stages.len(), 1);
        assert_eq!(p[0].stages[0].name, "ls");
        assert!(p[0].stages[0].args.is_empty());
        assert!(!p[0].background);
    }

    #[test]
    fn parse_command_with_args() {
        let p = parse("echo hello world");
        assert_eq!(p[0].stages[0].name, "echo");
        assert_eq!(p[0].stages[0].args, vec!["hello", "world"]);
    }

    #[test]
    fn parse_pipe_two_commands() {
        let p = parse("ls | grep rs");
        assert_eq!(p[0].stages.len(), 2);
        assert_eq!(p[0].stages[0].name, "ls");
        assert_eq!(p[0].stages[1].name, "grep");
        assert_eq!(p[0].stages[1].args, vec!["rs"]);
    }

    #[test]
    fn parse_pipe_three_commands() {
        let p = parse("a | b | c");
        assert_eq!(p[0].stages.len(), 3);
    }

    #[test]
    fn parse_redirect_out_truncate() {
        let p = parse("echo hi > out.txt");
        let r = p[0].stages[0].output_redirection.as_ref().unwrap();
        assert_eq!(r.path, "out.txt");
        assert!(!r.append);
    }

    #[test]
    fn parse_redirect_append() {
        let p = parse("echo hi >> out.txt");
        let r = p[0].stages[0].output_redirection.as_ref().unwrap();
        assert!(r.append);
    }

    #[test]
    fn parse_redirect_in() {
        let p = parse("wc -l < input.txt");
        assert_eq!(
            p[0].stages[0].input_redirection.as_deref(),
            Some("input.txt")
        );
    }

    #[test]
    fn parse_background_marker() {
        let p = parse("sleep 5 &");
        assert!(p[0].background);
        assert_eq!(p[0].stages[0].name, "sleep");
    }

    #[test]
    fn parse_quoted_string_preserves_spaces() {
        let p = parse("echo \"hello world\"");
        assert_eq!(p[0].stages[0].args, vec!["hello world"]);
    }

    #[test]
    fn parse_unterminated_quote_returns_err() {
        assert!(matches!(
            parse_pipelines("echo \"oops"),
            Err(ParseError::UnterminatedQuote)
        ));
    }

    #[test]
    fn parse_mixed_pipe_redirect_background() {
        let p = parse("cat < a | grep x > b &");
        assert!(p[0].background);
        assert_eq!(p[0].stages.len(), 2);
        assert_eq!(p[0].stages[0].input_redirection.as_deref(), Some("a"));
        assert_eq!(
            p[0].stages[1].output_redirection.as_ref().unwrap().path,
            "b"
        );
    }

    #[test]
    fn parse_multiple_statements_via_semicolon() {
        let p = parse("echo a ; echo b");
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].stages[0].args, vec!["a"]);
        assert_eq!(p[1].stages[0].args, vec!["b"]);
    }

    #[test]
    fn parse_missing_redirect_target_errors() {
        assert!(matches!(
            parse_pipelines("echo >"),
            Err(ParseError::MissingRedirectionTarget)
        ));
    }

    #[test]
    fn parse_empty_pipeline_stage_errors() {
        assert!(matches!(
            parse_pipelines("ls | | grep x"),
            Err(ParseError::EmptyPipelineStage)
        ));
    }

    #[test]
    fn expand_variable_simple() {
        let mut env = Environment::empty();
        env.set_var("HOME", "/home/u");
        assert_eq!(expand_variables("$HOME", &env), "/home/u");
    }

    #[test]
    fn expand_variable_undefined_becomes_empty() {
        let env = Environment::empty();
        assert_eq!(expand_variables("$NOPE/x", &env), "/x");
    }

    #[test]
    fn expand_variable_inside_word() {
        let mut env = Environment::empty();
        env.set_var("X", "ya");
        assert_eq!(expand_variables("pre$X post", &env), "preya post");
    }
}
