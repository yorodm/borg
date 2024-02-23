use crate::DirSettings;

use super::Builder;
use anyhow::{Context, Result};
use orgize::export::{DefaultHtmlHandler, SyntectHtmlHandler};
use orgize::Org;
use std::fs::File;

/// A wrapper around `DefaultHtmlHandler` in case I need to
/// customize some stuff
#[derive(Default)]
pub struct PublishHandler {
    dir_settings: DirSettings,
}

impl Builder<PublishHandler> for PublishHandler {
    fn build(&self) -> Result<()> {
        for f in &self.dir_settings.files {
            let contents = std::fs::read_to_string(f.path())?;
            let mut output_path = self.dir_settings.publish_dir.join(f.file_name());
            output_path.set_extension("html");
            let output = File::create(&output_path).context(format!(
                "Could not create output file {:?}",
                output_path.as_os_str()
            ))?;
            let mut inner = SyntectHtmlHandler::new(DefaultHtmlHandler);
            Org::parse(&contents)
                .write_html_custom(output, &mut inner)
                .context(format!("Failed to process {:?}", f.path().as_os_str()))?
        }
        Ok(())
    }

    fn from_project(project: &crate::Project) -> Result<Self> {
        Ok(PublishHandler {
            dir_settings: DirSettings::try_from(project)?,
        })
    }
}
