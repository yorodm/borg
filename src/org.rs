use crate::sitemap::{RssGenerator, SitemapGenerator};
use crate::{ensure_directory, DirSettings};

use super::Builder;
use anyhow::{Context, Error, Result};
use log::{error, info};
use orgize::export::{DefaultHtmlHandler, HtmlHandler, SyntectHtmlHandler};
use orgize::Org;
use std::fs::File;
use std::path::Path;

#[derive(Default)]
pub struct CustomHtmlHandler<'a>{
    html_handler: SyntectHtmlHandler<std::io::Error, DefaultHtmlHandler>,
    rss_handler : Option<RssGenerator>,
    sitemap_handler: Option<SitemapGenerator<'a>>
}

impl <'a> HtmlHandler<Error> for CustomHtmlHandler<'a> {
    fn start<W: std::io::Write>(
        &mut self,
        mut w: W,
        element: &orgize::Element,
    ) -> std::result::Result<(), Error> {
        match element {
            orgize::Element::Keyword(k) => {
                match k.key.as_ref() {
                    "time" | "TIME" => {
                        write!(& mut w, "<h1>{}</h1>", k.value)?;
                    },
                    "date" | "DATE" => {
                        write!(& mut w, "<span class=\"date\">{}</span>", k.value)?;
                    }
                    _ => todo!(),
                }
                Ok(())
            }
            _ => {
                self.html_handler.start(w, element)?;
                Ok(())
            }
        }
    }

    fn end<W: std::io::Write>(
        &mut self,
        w: W,
        element: &orgize::Element,
    ) -> std::result::Result<(), Error> {
        match element {
            orgize::Element::Keyword(k) => {
                Ok(())
            }
            _ => {
                self.html_handler.end(w, element)?;
                Ok(())
            }
        }
    }
}

/// Builder for the `ToHtml` action
#[derive(Default, Debug)]
pub struct PublishBuilder {
    dir_settings: DirSettings,
    rss_generator: RssGenerator,
    sitemap_generator: SitemapGenerator
}

impl Builder<PublishBuilder> for PublishBuilder {
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
        Ok(PublishBuilder {
            dir_settings: DirSettings::new_from_project(project, root)?,
        })
    }
}

impl PublishBuilder {
    fn export_html(&self, f: &walkdir::DirEntry) -> Result<(), Error> {
        let contents = std::fs::read_to_string(f.path())?;
        let mut output_path = self.dir_settings.publish_dir.join(f.file_name());
        output_path.set_extension("html");
        info!(
            "Processing file {} into {:?}",
            f.path().display(),
            output_path.display()
        );
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
        let mut inner = CustomHtmlHandler::default();
        Ok(Org::parse(&contents)
            .write_html_custom(output, &mut inner)
            .context(format!("Failed to process {:?}", f.path().display()))
            .map_err(|e| {
                error!("{}", e);
                e
            })?)
    }
}
