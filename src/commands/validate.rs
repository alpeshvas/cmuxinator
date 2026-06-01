use anyhow::Result;

use crate::{cmux, config, formatting};

pub fn run(project: &str) -> Result<()> {
    let loaded = config::load_project(project)?;
    loaded.project.validate()?;
    cmux::ensure_available()?;
    formatting::print_validation_success(&loaded.project.name, &loaded.path);
    Ok(())
}
