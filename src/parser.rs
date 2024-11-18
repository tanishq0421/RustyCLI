use crate::command::{Command, Operator};
use crate::env_vars::Environment;
use regex::Regex;

pub fn expand_variables(input: &str, env: &Environment) -> String {
    let re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        env.get_var(&caps[1]).unwrap_or("").to_string()
    })
    .to_string()
}

pub fn parse_input(input: &str) -> Vec<Command> {
    let mut commands = Vec::new();
    let tokens = tokenize(input);
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        let mut command = Command {
            name: token,
            args: Vec::new(),
            operator: Operator::None,
            next: None,
            input_redirection: None,
            output_redirection: None,
            append_output: false,
            background: false,
        };

        while let Some(next_token) = iter.peek() {
            match next_token.as_str() {
                "|" => {
                    iter.next();
                    command.operator = Operator::Pipe;
                    command.next = Some(Box::new(parse_command(&mut iter)));
                    break;
                }
                "<" => {
                    iter.next();
                    if let Some(file) = iter.next() {
                        command.input_redirection = Some(file);
                    }
                }
                ">" => {
                    iter.next();
                    if let Some(file) = iter.next() {
                        command.output_redirection = Some(file);
                        command.append_output = false;
                    }
                }
                ">>" => {
                    iter.next();
                    if let Some(file) = iter.next() {
                        command.output_redirection = Some(file);
                        command.append_output = true;
                    }
                }
                "&" => {
                    iter.next();
                    command.background = true;
                    break;
                }
                _ => {
                    command.args.push(iter.next().unwrap());
                }
            }
        }
        commands.push(command);
    }
    commands
}

fn parse_command(iter: &mut std::iter::Peekable<std::vec::IntoIter<String>>) -> Command {
    let mut command = Command {
        name: iter.next().unwrap_or_default(),
        args: Vec::new(),
        operator: Operator::None,
        next: None,
        input_redirection: None,
        output_redirection: None,
        append_output: false,
        background: false,
    };

    while let Some(next_token) = iter.peek() {
        match next_token.as_str() {
            "|" => {
                iter.next();
                command.operator = Operator::Pipe;
                command.next = Some(Box::new(parse_command(iter)));
                break;
            }
            "<" => {
                iter.next();
                if let Some(file) = iter.next() {
                    command.input_redirection = Some(file);
                }
            }
            ">" => {
                iter.next();
                if let Some(file) = iter.next() {
                    command.output_redirection = Some(file);
                    command.append_output = false;
                }
            }
            ">>" => {
                iter.next();
                if let Some(file) = iter.next() {
                    command.output_redirection = Some(file);
                    command.append_output = true;
                }
            }
            "&" => {
                iter.next();
                command.background = true;
                break;
            }
            _ => {
                command.args.push(iter.next().unwrap());
            }
        }
    }
    command
}

fn tokenize(input: &str) -> Vec<String> {
    let re = Regex::new(r#"(?P<token>\|{1,2}|&{1}|>{1,2}|<|[^ \t\n\r\f\v]+)"#).unwrap();
    re.captures_iter(input)
        .map(|cap| cap["token"].to_string())
        .collect()
}
