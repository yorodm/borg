use super::Builder;
use anyhow::{anyhow, Context, Result};
use globwalk::{DirEntry, FileType, GlobWalkerBuilder};
use orgize::export::{DefaultHtmlHandler, SyntectHtmlHandler};
use orgize::Org
use std::fs::File;
use std::path::{Path, PathBuf};

/// A wrapper around `DefaultHtmlHandler` in case I need to
/// customize some stuff
#[derive(Default)]
pub struct PublishHandler {
    file_list: Vec<DirEntry>,
    dest_dir: PathBuf
}

impl Builder for PublishHandler {
    fn build(&self) -> Result<()> {
        for f in &self.file_list {
            let contents = std::fs::read_to_string(f.path())?;
            let mut output_path = self.dest_dir.join(f.file_name());
            output_path.set_extension("html");
            let output = File::create(&output_path)
                .context(format!("Could not create output file {:?}", output_path.as_os_str()))?;
            let mut inner = SyntectHtmlHandler::new(DefaultHtmlHandler);
            Org::parse(&contents)
                .write_html_custom(output, &mut inner)
                .context(format!("Failed to process {:?}", f.path().as_os_str()))?
        }
        Ok(())
    }

    fn from_project(project: crate::Project) -> Result<Self> {
        if let Some(path) = project.base_directory {
            if Path::new(&path).is_dir() {
                let entries: Vec<DirEntry> = GlobWalkerBuilder::from_patterns(
                    path,
                    &vec![&project.base_extension.unwrap_or_else(|| "*".into())],
                )
                .follow_links(true)
                .max_depth(project.recursive)
                .file_type(FileType::FILE)
                .build()?
                .into_iter()
                .filter_map(Result::ok)
                    .collect();
                return Ok(PublishHandler {
                    file_list: entries,
                    dest_dir: PathBuf::from(project.publishing_directory)
                });
            }
        }
        Err(anyhow!(
            "Base directory does not exists or hasn't been defined"
        ))
    }
}
