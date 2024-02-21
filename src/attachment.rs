use std::path::PathBuf;
use anyhow::Result;

use crate::Builder;

pub struct StaticsHandler {
    source_dir: PathBuf,
    dest_dir: PathBuf,
}


impl Builder for StaticsHandler {
    fn build(&self) -> Result<()> {
        todo!()
    }

    fn from_project(project: crate::Project) -> Result<Self> {
        todo!()
    }
}
