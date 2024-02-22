use anyhow::Result;
use walkdir::DirEntry;
use std::path::PathBuf;

use crate::Builder;

pub struct AttachmentsHandler {
    file_list: Vec<DirEntry>,
    dest_dir: PathBuf,
}

impl Builder for AttachmentsHandler {
    fn build(&self) -> Result<()> {
        todo!()
    }

    fn from_project(project: &crate::Project) -> Result<Self> {
        todo!()
    }
}
