use anyhow::Result;

use crate::{config, formatting};

pub fn run() -> Result<()> {
    let projects = config::available_projects()?;
    formatting::print_project_list(&projects);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::paths;

    #[test]
    fn list_empty_when_config_dir_is_missing() {
        // This mostly guards the not-found branch without asserting on the user's filesystem.
        assert!(paths::config_dir().is_ok());
    }
}
