use std::ffi::OsStr;

use clap_complete::engine::CompletionCandidate;

use crate::config;

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

fn current_project_from_completion_args() -> Option<String> {
    let mut args = std::env::args_os().skip_while(|arg| arg != "--");
    args.next()?;

    let words = args
        .filter_map(|arg| arg.into_string().ok())
        .collect::<Vec<_>>();

    project_from_completion_words(&words)
}

fn project_from_completion_words(words: &[String]) -> Option<String> {
    let command_index = words
        .iter()
        .position(|word| matches!(word.as_str(), "start" | "dry-run"))?;

    words[command_index + 1..]
        .iter()
        .filter(|word| !matches!(word.as_str(), "--no-group" | "--help" | "-h"))
        .find(|word| !word.is_empty() && !word.starts_with('-'))
        .cloned()
}

fn complete_values(current: &OsStr, values: Vec<String>) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };

    values
        .into_iter()
        .filter(|value| value.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
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
        let words = ["cmuxinator", "start", "--no-group", "dex", ""]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(
            project_from_completion_words(&words).as_deref(),
            Some("dex")
        );
    }
}
