use super::Builder;
use anyhow::Result;
use orgize::export::{DefaultHtmlHandler, HtmlHandler};
use orgize::{Element, Org};
use std::convert::From;
use std::io::{Error as IOError, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct HandlerError;

impl From<IOError> for HandlerError {
    fn from(value: IOError) -> Self {
        todo!()
    }
}

/// A wrapper around `DefaultHtmlHandler` in case I need to
/// customize some stuff
#[derive(Default)]
pub struct BorgHandler(DefaultHtmlHandler);

impl HtmlHandler<HandlerError> for BorgHandler {
    fn start<W: Write>(&mut self, w: W, element: &Element) -> Result<(), HandlerError> {
        self.0.start(w, element)?;
        Ok(())
    }

    fn end<W: Write>(&mut self, w: W, element: &Element) -> Result<(), HandlerError> {
        self.0.end(w, element)?;
        Ok(())
    }
}

impl Builder for BorgHandler {}

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

impl Builder for StaticsHandler{

}
