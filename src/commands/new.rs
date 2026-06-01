use std::fs;

use anyhow::{Context, Result, anyhow};

use crate::{paths, templates};

pub fn run(project: &str, force: bool) -> Result<()> {
    paths::validate_project_slug(project)?;
    paths::ensure_config_dir()?;

    let existing_path = paths::project_path(project)?;
    let path = if existing_path.exists() {
        existing_path
    } else {
        paths::new_project_path(project)?
    };

    if path.exists() && !force {
        return Err(anyhow!(
            "project file already exists at {}; pass --force to overwrite",
            path.display()
        ));
    }

    fs::write(&path, templates::project_template(project))
        .with_context(|| format!("failed to write project template to {}", path.display()))?;

    println!("Created {}", path.display());
    Ok(())
}
