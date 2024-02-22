use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{Builder, DirSettings};

pub struct AttachmentsHandler {
    dir_settings: DirSettings,
}

pub fn copy_creating_dirs<P: AsRef<Path>>(source: P, output_dir: P, dest: P) -> Result<()> {
    Ok(())
}

impl Builder for AttachmentsHandler {
    fn build(&self) -> Result<()> {
        for entry in self.dir_settings.files.iter() {
            // this will f
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
