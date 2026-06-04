use std::{ffi::OsStr, io::Write};

use clap_complete::engine::CompletionCandidate;

use crate::config;

#[derive(Debug, PartialEq, Eq)]
enum CompletionSlot {
    Project,
    Workspace(String),
    NoValues,
}

pub fn project_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    complete_values(current, config::available_projects().unwrap_or_default())
}

pub fn workspace_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let values = match current_project_from_completion_args() {
        Some(project) => config::workspace_names_for_project(&project).unwrap_or_default(),
        None => config::available_workspace_names().unwrap_or_default(),
    };

    complete_values(current, values)
}

pub fn try_complete_project_or_workspace() -> std::io::Result<bool> {
    if std::env::var_os("COMPLETE").is_none() {
        return Ok(false);
    }

    let Some((words, current_index)) = completion_words() else {
        return Ok(false);
    };
    let Some(values) = project_or_workspace_values(&words, current_index) else {
        return Ok(false);
    };

    write_values(&values)?;
    Ok(true)
}

fn project_or_workspace_values(words: &[String], current_index: usize) -> Option<Vec<String>> {
    let current = words.get(current_index).map(String::as_str).unwrap_or("");

    match completion_slot(words, current_index)? {
        CompletionSlot::Project => Some(filter_values(
            current,
            config::available_projects().unwrap_or_default(),
        )),
        CompletionSlot::Workspace(project) => Some(filter_values(
            current,
            config::workspace_names_for_project(&project).unwrap_or_default(),
        )),
        CompletionSlot::NoValues => Some(Vec::new()),
    }
}

fn completion_slot(words: &[String], current_index: usize) -> Option<CompletionSlot> {
    let current = words.get(current_index).map(String::as_str).unwrap_or("");
    if current.starts_with('-') {
        return None;
    }

    let command_index = words
        .iter()
        .position(|word| matches!(word.as_str(), "start" | "dry-run" | "validate"))?;
    if current_index <= command_index {
        return None;
    }

    let project = project_before_current_word(words, command_index, current_index);

    match words[command_index].as_str() {
        "validate" => Some(if project.is_some() {
            CompletionSlot::NoValues
        } else {
            CompletionSlot::Project
        }),
        "start" | "dry-run" => Some(match project {
            Some(project) => CompletionSlot::Workspace(project),
            None => CompletionSlot::Project,
        }),
        _ => None,
    }
}

fn project_before_current_word(
    words: &[String],
    command_index: usize,
    current_index: usize,
) -> Option<String> {
    words
        .iter()
        .take(current_index)
        .skip(command_index + 1)
        .find(|word| is_positional_value(word))
        .cloned()
}

fn is_positional_value(word: &str) -> bool {
    !word.is_empty() && !word.starts_with('-')
}

fn completion_words() -> Option<(Vec<String>, usize)> {
    let current_index = std::env::var("_CLAP_COMPLETE_INDEX").ok()?.parse().ok()?;
    let mut args = std::env::args_os().skip_while(|arg| arg != "--");
    args.next()?;

    let mut words = Vec::new();
    for arg in args {
        words.push(arg.into_string().ok()?);
    }

    Some((words, current_index))
}

fn current_project_from_completion_args() -> Option<String> {
    let (words, _) = completion_words()?;
    project_from_completion_words(&words)
}

fn project_from_completion_words(words: &[String]) -> Option<String> {
    let command_index = words
        .iter()
        .position(|word| matches!(word.as_str(), "start" | "dry-run"))?;

    words[command_index + 1..]
        .iter()
        .find(|word| is_positional_value(word))
        .cloned()
}

fn complete_values(current: &OsStr, values: Vec<String>) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };

    filter_values(current, values)
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

fn filter_values(current: &str, values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter(|value| value.starts_with(current))
        .collect()
}

fn write_values(values: &[String]) -> std::io::Result<()> {
    let separator = std::env::var("_CLAP_IFS").unwrap_or_else(|_| "\n".to_string());
    let mut stdout = std::io::stdout();

    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            stdout.write_all(separator.as_bytes())?;
        }
        stdout.write_all(value.as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_completion_candidates_by_prefix() {
        let candidates = complete_values(OsStr::new("de"), vec!["dex".into(), "pi".into()]);
        let values = candidates
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(values, vec!["dex"]);
    }

    #[test]
    fn finds_project_for_workspace_completion() {
        let words = ["cmuxinator", "start", "--group", "dex", ""]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(
            project_from_completion_words(&words).as_deref(),
            Some("dex")
        );
    }

    #[test]
    fn treats_empty_start_position_as_project_slot() {
        let words = ["cmuxinator", "start", ""]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(completion_slot(&words, 2), Some(CompletionSlot::Project));
    }

    #[test]
    fn treats_position_after_project_as_workspace_slot() {
        let words = ["cmuxinator", "start", "--group", "dex", ""]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(
            completion_slot(&words, 4),
            Some(CompletionSlot::Workspace("dex".into()))
        );
    }

    #[test]
    fn lets_clap_handle_option_completion() {
        let words = ["cmuxinator", "start", "--"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(completion_slot(&words, 2), None);
    }
}
