use super::Builder;
use anyhow::Result;
use orgize::export::{DefaultHtmlHandler, SyntectHtmlHandler, HtmlHandler};
use orgize::{Element, Org};
use std::io::{Error as IOError, Write};
use std::path::{Path, PathBuf};

/// A wrapper around `DefaultHtmlHandler` in case I need to
/// customize some stuff
#[derive(Default)]
pub struct PublishHandler{
    inner: SyntectHtmlHandler<IOError, DefaultHtmlHandler>,
}

impl PublishHandler {
    fn from_file<P: AsRef<Path>>(org_file: P) -> Self {
        todo!()
    }
}

impl Builder for PublishHandler {
    fn build(project: crate::Project) -> Result<()> {
        project.
    }
}

pub struct StaticsHandler {
    source_dir: PathBuf,
    dest_dir: PathBuf,
}

impl StaticsHandler {
    pub fn new<P: AsRef<Path>>(source: P, dest: P) -> Result<Self> {
        Ok(StaticsHandler {
            source_dir: source.as_ref().into(),
            dest_dir: dest.as_ref().into(),
        })
    }
}

impl Builder for StaticsHandler {
    fn build(project: crate::Project) -> Result<()> {
        todo!()
    }
}
