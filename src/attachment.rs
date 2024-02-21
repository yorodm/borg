use anyhow::Result;
use std::path::PathBuf;

use crate::Builder;

pub struct AttachmentsHandler {
    source_dir: PathBuf,
    dest_dir: PathBuf,
}

impl Builder for AttachmentsHandler {
    fn build(&self) -> Result<()> {
        todo!()
    }

    fn from_project(project: crate::Project) -> Result<Self> {
        todo!()
    }
}
