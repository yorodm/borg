use crate::DirSettings;

use super::Builder;
use anyhow::{Context, Result};
use orgize::export::{DefaultHtmlHandler, SyntectHtmlHandler};
use orgize::Org;
use std::fs::File;

/// Handler for the `ToHtml` action
#[derive(Default)]
pub struct PublishHandler {
    dir_settings: DirSettings,
}

impl Builder<PublishHandler> for PublishHandler {
    fn build(&self) -> Result<()> {
        for f in self.dir_settings.files.iter() {
            self.export_html(f)?;
        }
        Ok(())
    }

    fn from_project(project: &crate::Project) -> Result<Self> {
        Ok(PublishHandler {
            dir_settings: DirSettings::try_from(project)?,
        })
    }
}

impl PublishHandler {
    // TODO: This has to work with recursive files too
    fn export_html(&self, f: &walkdir::DirEntry) -> Result<(), anyhow::Error> {
        let contents = std::fs::read_to_string(f.path())?;
        let mut output_path = self.dir_settings.publish_dir.join(f.file_name());
        output_path.set_extension("html");
        let output = File::create(&output_path).context(format!(
            "Could not create output file {:?}",
            output_path.as_os_str()
        ))?;
        let mut inner = SyntectHtmlHandler::new(DefaultHtmlHandler);
        Ok(Org::parse(&contents)
            .write_html_custom(output, &mut inner)
            .context(format!("Failed to process {:?}", f.path().as_os_str()))?)
    }
}
