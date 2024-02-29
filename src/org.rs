use crate::{DirSettings, ensure_directory};

use super::Builder;
use anyhow::{Context, Result};
use log::{error, info};
use orgize::export::{DefaultHtmlHandler, SyntectHtmlHandler};
use orgize::Org;
use std::fs::File;
use std::path::Path;

/// Handler for the `ToHtml` action
#[derive(Default, Debug)]
pub struct PublishHandler {
    dir_settings: DirSettings,
}


impl Builder<PublishHandler> for PublishHandler {
    fn build(&self) -> Result<()> {
        ensure_directory(&self.dir_settings.publish_dir)?;
        for f in self.dir_settings.files.iter() {
            self.export_html(f)
                .context(format!("Error exporting file {}", f.path().display()))
                .map_err(|e| {
                    error!("{}", e);
                    e
                })?;
        }
        Ok(())
    }

    fn from_project<P: AsRef<Path>>(project: &crate::Project, root: P) -> Result<Self> {
        Ok(PublishHandler {
            dir_settings: DirSettings::new_from_project(project, root)?
        })
    }
}

impl PublishHandler {
    // TODO: This has to work with recursive files too
    fn export_html(&self, f: &walkdir::DirEntry) -> Result<(), anyhow::Error> {
        let contents = std::fs::read_to_string(f.path())?;
        let mut output_path = self.dir_settings.publish_dir.join(f.file_name());
        output_path.set_extension("html");
        info!("Processing file {} into {:?}", f.path().display(), output_path.display());
        let output = File::create(&output_path)
            .map_err(|e| {
                error!("{}", e);
                e
            })
            .context(format!(
                "Could not create output file {:?}",
                output_path.display(),
            ))
            .map_err(|e| {
                error!("{}", e);
                e
            })?;
        let mut inner = SyntectHtmlHandler::new(DefaultHtmlHandler);
        Ok(Org::parse(&contents)
            .write_html_custom(output, &mut inner)
            .context(format!("Failed to process {:?}", f.path().display()))
            .map_err(|e| {
                error!("{}", e);
                e
            })?)
    }
}
