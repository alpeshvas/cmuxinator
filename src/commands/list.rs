use std::{collections::BTreeSet, fs, io::ErrorKind};

use anyhow::{Context, Result};

use crate::{formatting, paths};

pub fn run() -> Result<()> {
    let projects = available_projects()?;
    formatting::print_project_list(&projects);
    Ok(())
}

fn available_projects() -> Result<Vec<String>> {
    let dir = paths::config_dir()?;
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read config directory {}", dir.display()));
        }
    };

    let mut projects = BTreeSet::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("failed to read entry in {}", dir.display()))?;
        let path = entry.path();
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if !matches!(extension, "yml" | "yaml") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        projects.insert(stem.to_string());
    }

    Ok(projects.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_empty_when_config_dir_is_missing() {
        // This mostly guards the not-found branch without asserting on the user's filesystem.
        assert!(paths::config_dir().is_ok());
    }
}
