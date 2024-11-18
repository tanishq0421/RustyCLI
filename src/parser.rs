use crate::env_vars::Environment;
use regex::Regex;

pub fn expand_variables(input: &str, env: &Environment) -> String {
    let re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        env.get_var(&caps[1]).unwrap_or("").to_string()
    })
    .to_string()
}
