use std::{
    fs::{copy, create_dir_all},
    path::Path,
};

use anyhow::{Context, Result};

use crate::{Builder, DirSettings};

pub struct AttachmentsHandler {
    dir_settings: DirSettings,
}

fn copy_creating_dirs<P: AsRef<Path>>(source: P, output_dir: P, dest: P) -> Result<()> {
    if let Some(path) = dest.as_ref().parent() {
        let new_dir = output_dir.as_ref().join(path);
        if !new_dir.exists() {
            create_dir_all(new_dir)?
        }
    };
    copy(&source, output_dir.as_ref().join(dest.as_ref())).context(format!(
        "Error copying {} to {}",
        source.as_ref().to_string_lossy(),
        output_dir.as_ref().to_string_lossy()
    ))?;
    Ok(())
}

impl Builder for AttachmentsHandler {
    fn build(&self) -> Result<()> {
        for entry in self.dir_settings.files.iter() {
            let dest = entry.path().strip_prefix(&self.dir_settings.source_dir)?;
            copy_creating_dirs(entry.path(), &self.dir_settings.publish_dir, dest)?;
        }
        Ok(())
    }

    fn from_project(project: &crate::Project) -> Result<Self> {
        Ok(AttachmentsHandler {
            dir_settings: DirSettings::try_from(project)?,
        })
    }
}
