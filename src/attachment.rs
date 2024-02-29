use std::{
    fs::{copy, create_dir_all},
    path::Path,
};

use log::error;

use anyhow::{Context, Result};

use crate::{Builder, DirSettings};

#[derive(Debug)]
pub struct AttachmentsHandler {
    dir_settings: DirSettings,
}

fn copy_creating_dirs<P: AsRef<Path>>(source: P, output_dir: P, dest: P) -> Result<()> {
    if let Some(path) = dest.as_ref().parent() {
        let new_dir = output_dir.as_ref().join(path);
        if !new_dir.exists() {
            create_dir_all(new_dir).map_err(|e| {
                error!("{}", e);
                e
            })?
        }
    };
    copy(&source, output_dir.as_ref().join(dest.as_ref()))
        .map_err(|e| {
            error!("{}", e);
            e
        })
        .context(format!(
            "Error copying {} to {}",
            source.as_ref().display(),
            output_dir.as_ref().display()
        ))
        .map_err(|e| {
            error!("{}", e);
            e
        })?;
    Ok(())
}

impl Builder<AttachmentsHandler> for AttachmentsHandler {
    fn build(&self) -> Result<()> {
        for entry in self.dir_settings.files.iter() {
            let canonical_entry = entry.path().canonicalize().map_err(|e| {
                error!(
                    "Error {}: File {} seems to be gone!",
                    e,
                    entry.path().display()
                );
                e
            })?;
            let dest = &canonical_entry
                .strip_prefix(&self.dir_settings.source_dir)
                .map_err(|e| {
                    error!("{}", e);
                    e
                })?;
            copy_creating_dirs(entry.path(), &self.dir_settings.publish_dir, dest)?;
        }
        Ok(())
    }

    fn from_project<P: AsRef<Path>>(project: &crate::Project, root: P) -> Result<Self> {
        Ok(AttachmentsHandler {
            dir_settings: DirSettings::new_from_project(project, root)?,
        })
    }
}
