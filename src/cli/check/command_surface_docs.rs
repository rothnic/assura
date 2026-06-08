//! Command-surface documentation validation for first-party custom constraints.

use super::CheckError;
use crate::config::config::{CommandSurfaceCommand, CommandSurfaceContract};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub(super) struct CommandSurfaceProblem {
    pub(super) example: String,
    pub(super) message: String,
}

pub(super) fn load_command_surface_contract(
    path: &Path,
) -> Result<CommandSurfaceContract, CheckError> {
    let content = fs::read_to_string(path)?;
    let contract: CommandSurfaceContract = serde_yaml::from_str(&content).map_err(|error| {
        CheckError::Config(crate::cli::config::ConfigError::Invalid(format!(
            "invalid command surface contract {}: {error}",
            path.display()
        )))
    })?;
    validate_contract(&contract, path)?;
    Ok(contract)
}

pub(super) fn command_surface_problems(
    contract: &CommandSurfaceContract,
    content: &str,
) -> Vec<CommandSurfaceProblem> {
    let mut problems = Vec::new();
    for example in documented_command_examples(content) {
        let Some(invocation) = AssuraInvocation::parse(&example) else {
            continue;
        };
        problems.extend(
            contract
                .problems_for(&invocation)
                .into_iter()
                .map(|message| CommandSurfaceProblem {
                    example: invocation.original.clone(),
                    message,
                }),
        );
    }
    problems
}

#[derive(Debug)]
struct AssuraInvocation {
    original: String,
    tokens: Vec<String>,
}

impl AssuraInvocation {
    fn parse(example: &str) -> Option<Self> {
        let tokens = shell_words(example);
        let tokens = normalize_assura_tokens(&tokens)?;
        if tokens.len() < 2 || tokens.first()? != "assura" {
            return None;
        }
        Some(Self {
            original: example.trim().to_string(),
            tokens,
        })
    }
}

impl CommandSurfaceContract {
    fn problems_for(&self, invocation: &AssuraInvocation) -> Vec<String> {
        let Some(command) = self
            .commands
            .iter()
            .filter_map(|command| command_prefix_len(command, invocation).map(|len| (command, len)))
            .max_by_key(|(_, len)| *len)
        else {
            return vec![format!(
                "references unsupported command family `{}`",
                invocation
                    .tokens
                    .iter()
                    .take(2)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ")
            )];
        };
        command.0.problems_for(&invocation.tokens[command.1..])
    }
}

impl CommandSurfaceCommand {
    fn problems_for(&self, args: &[String]) -> Vec<String> {
        let mut problems = Vec::new();
        let lookup = flag_lookup(self);
        let mut seen_flags = HashMap::new();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            if !arg.starts_with('-') {
                if !self.allow_positionals {
                    problems.push(format!("uses unsupported positional argument `{arg}`"));
                }
                index += 1;
                continue;
            }

            let (flag_name, inline_value) = split_flag_value(arg);
            let Some(canonical) = lookup.get(flag_name).copied() else {
                problems.push(format!("uses unsupported flag `{flag_name}`"));
                index += 1;
                continue;
            };
            let flag = self.flags.get(canonical).expect("canonical flag exists");
            if flag.takes_value {
                let value = inline_value
                    .map(str::to_string)
                    .or_else(|| args.get(index + 1).cloned());
                if let Some(value) = value {
                    if !flag.values.is_empty() && !flag.values.contains(&value) {
                        problems.push(format!(
                            "uses unsupported value `{value}` for flag `{canonical}`"
                        ));
                    }
                    seen_flags.insert(canonical.to_string(), value);
                    if inline_value.is_none() {
                        index += 1;
                    }
                } else {
                    problems.push(format!("omits required value for flag `{canonical}`"));
                }
            } else if inline_value.is_some() {
                problems.push(format!("passes a value to valueless flag `{canonical}`"));
            } else {
                seen_flags.insert(canonical.to_string(), String::new());
            }
            index += 1;
        }
        for (name, flag) in &self.flags {
            if !seen_flags.contains_key(name) {
                continue;
            }
            for (required_flag, required_value) in &flag.requires {
                if seen_flags.get(required_flag) != Some(required_value) {
                    problems.push(format!(
                        "requires flag `{required_flag}` to be `{required_value}` when `{name}` is used"
                    ));
                }
            }
        }
        problems
    }
}

fn command_prefix_len(
    command: &CommandSurfaceCommand,
    invocation: &AssuraInvocation,
) -> Option<usize> {
    let command_tokens = command.name.split_whitespace().collect::<Vec<_>>();
    invocation
        .tokens
        .iter()
        .map(String::as_str)
        .zip(command_tokens.iter().copied())
        .all(|(actual, expected)| actual == expected)
        .then_some(command_tokens.len())
        .filter(|len| invocation.tokens.len() >= *len)
}

fn flag_lookup(command: &CommandSurfaceCommand) -> HashMap<&str, &str> {
    let mut lookup = HashMap::new();
    for (name, flag) in &command.flags {
        lookup.insert(name.as_str(), name.as_str());
        for alias in &flag.aliases {
            lookup.insert(alias.as_str(), name.as_str());
        }
    }
    lookup
}

fn validate_contract(contract: &CommandSurfaceContract, path: &Path) -> Result<(), CheckError> {
    let mut command_names = HashSet::new();
    for command in &contract.commands {
        if command.name.trim().is_empty() {
            return invalid_contract(path, "command name must not be empty");
        }
        if !command_names.insert(command.name.as_str()) {
            return invalid_contract(path, &format!("duplicate command `{}`", command.name));
        }

        let mut seen_flags = HashMap::new();
        for (name, flag) in &command.flags {
            if !name.starts_with('-') {
                return invalid_contract(
                    path,
                    &format!("flag `{name}` in `{}` must start with '-'", command.name),
                );
            }
            if seen_flags.insert(name.as_str(), name.as_str()).is_some() {
                return invalid_contract(path, &format!("duplicate flag `{name}`"));
            }
            for alias in &flag.aliases {
                if !alias.starts_with('-') {
                    return invalid_contract(
                        path,
                        &format!("alias `{alias}` in `{}` must start with '-'", command.name),
                    );
                }
                if let Some(existing) = seen_flags.insert(alias.as_str(), name.as_str()) {
                    return invalid_contract(
                        path,
                        &format!(
                            "alias `{alias}` in `{}` collides with `{existing}`",
                            command.name
                        ),
                    );
                }
            }
            for required_flag in flag.requires.keys() {
                if !command.flags.contains_key(required_flag) {
                    return invalid_contract(
                        path,
                        &format!(
                            "flag `{name}` in `{}` requires unknown flag `{required_flag}`",
                            command.name
                        ),
                    );
                }
            }
        }
    }
    Ok(())
}

fn invalid_contract<T>(path: &Path, message: &str) -> Result<T, CheckError> {
    Err(CheckError::Config(
        crate::cli::config::ConfigError::Invalid(format!(
            "invalid command surface contract {}: {message}",
            path.display()
        )),
    ))
}

fn split_flag_value(arg: &str) -> (&str, Option<&str>) {
    arg.split_once('=')
        .map(|(flag, value)| (flag, Some(value)))
        .unwrap_or((arg, None))
}

fn normalize_assura_tokens(tokens: &[String]) -> Option<Vec<String>> {
    let first = tokens.first()?;
    if first == "assura" || first.ends_with("/assura") {
        let mut command = tokens.to_vec();
        command[0] = "assura".to_string();
        return Some(command);
    }

    if tokens.first()? != "cargo" || tokens.get(1)? != "run" {
        return None;
    }
    let separator = tokens.iter().position(|token| token == "--")?;
    let mut command = tokens[separator + 1..].to_vec();
    if command.first().is_some_and(|token| token == "assura") {
        Some(command)
    } else {
        command.insert(0, "assura".to_string());
        Some(command)
    }
}

fn documented_command_examples(content: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            if let Some(example) = shell_example_line(trimmed) {
                examples.push(example);
            }
        }
        examples.extend(inline_code_examples(trimmed));
    }
    examples
}

fn shell_example_line(line: &str) -> Option<String> {
    let line = line.strip_prefix('$').unwrap_or(line).trim();
    let line = line.strip_prefix('>').unwrap_or(line).trim();
    let line = line.trim_end_matches('\\').trim();
    (!line.starts_with('#') && looks_like_command_example(line)).then(|| line.to_string())
}

fn inline_code_examples(line: &str) -> Vec<String> {
    if is_negative_or_future_example_line(line) {
        return Vec::new();
    }
    let mut examples = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find('`') else {
            break;
        };
        let part = &after_open[..close];
        if looks_like_command_example(part) {
            examples.push(part.to_string());
        }
        rest = &after_open[close + 1..];
    }
    examples
}

fn looks_like_command_example(value: &str) -> bool {
    let tokens = shell_words(value);
    !tokens.iter().any(|token| token.contains("...")) && normalize_assura_tokens(&tokens).is_some()
}

fn is_negative_or_future_example_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "failing:",
        "rejected",
        "reintroduces",
        "not supported",
        "unsupported",
        "older ideas",
        "historical",
        "future",
        "later",
        "not a command",
        "stale",
        "old ",
        "no `",
        "do not imply",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn shell_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in input.chars() {
        if quote.is_none() && matches!(ch, ';' | '|' | '&') {
            break;
        }
        match ch {
            '\'' | '"' if quote == Some(ch) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(ch),
            ch if quote.is_none() && ch.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            ch => current.push(ch),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}
