use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};

pub fn config_dir() -> Result<PathBuf> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME")
        && !config_home.trim().is_empty()
    {
        return Ok(PathBuf::from(config_home).join("cmuxinator"));
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow!("could not determine home directory"))?;
    Ok(home.join(".config").join("cmuxinator"))
}

pub fn ensure_config_dir() -> Result<PathBuf> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create config directory {}", dir.display()))?;
    Ok(dir)
}

pub fn project_path(project: &str) -> Result<PathBuf> {
    let expanded = expand_tilde(project);
    let raw = Path::new(&expanded);

    if looks_like_path(raw, project) {
        return Ok(PathBuf::from(expanded));
    }

    let dir = config_dir()?;
    let yml = dir.join(format!("{project}.yml"));
    if yml.exists() {
        return Ok(yml);
    }

    let yaml = dir.join(format!("{project}.yaml"));
    if yaml.exists() {
        return Ok(yaml);
    }

    Ok(yml)
}

pub fn new_project_path(project: &str) -> Result<PathBuf> {
    validate_project_slug(project)?;
    Ok(config_dir()?.join(format!("{project}.yml")))
}

pub fn expand_cwd(cwd: &str) -> Result<String> {
    let expanded = expand_tilde(cwd);
    if expanded.trim().is_empty() {
        return Err(anyhow!("cwd cannot be empty"));
    }
    if expanded.contains('\0') {
        return Err(anyhow!("cwd contains a NUL byte"));
    }
    Ok(expanded)
}

pub fn expand_tilde(path: &str) -> String {
    shellexpand::tilde(path).into_owned()
}

pub fn validate_project_slug(project: &str) -> Result<()> {
    validate_name(project, "project")?;
    if project.contains('/') || project.contains('\\') {
        return Err(anyhow!(
            "project name must not contain path separators: {project}"
        ));
    }
    if project == "." || project == ".." {
        return Err(anyhow!("project name cannot be {project:?}"));
    }
    if project.ends_with(".yml") || project.ends_with(".yaml") {
        return Err(anyhow!(
            "project name should not include a file extension: {project}"
        ));
    }
    Ok(())
}

pub fn validate_name(name: &str, kind: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow!("{kind} name cannot be empty"));
    }
    if name.contains('\n') || name.contains('\r') {
        return Err(anyhow!("{kind} name cannot contain newlines: {name:?}"));
    }
    if name.contains('\0') {
        return Err(anyhow!("{kind} name cannot contain a NUL byte"));
    }
    Ok(())
}

fn looks_like_path(path: &Path, original: &str) -> bool {
    path.is_absolute()
        || original.starts_with('~')
        || original.contains('/')
        || original.contains('\\')
        || matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_project_slugs_with_extensions() {
        assert!(validate_project_slug("dex.yml").is_err());
    }

    #[test]
    fn expands_tilde_paths() {
        let expanded = expand_cwd("~/example").unwrap();
        assert!(!expanded.starts_with('~'));
    }
}
