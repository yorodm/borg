use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub mod org;

#[derive(Debug, Serialize, Deserialize)]
pub enum PublishAction {
    ToHtml,
    Attachment,
    Rss
}

impl Default for PublishAction {
    fn default() -> Self {
        Self::Attachment // By default we only copy
    }
}

/// A site config
/// Basically a list of `Project` definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    projects: Vec<Project>
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        todo!()
    }
}

/// A publishing project.
/// Made this as close as possible to org-publish projects while still
/// considering my own use case
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Project {
    base_directory: Option<String>,
    base_extension: Option<String>,
    recursive: bool,
    publishing_directory: String,
    exclude: Vec<String>,
    auto_sitemap: bool,
    sitemap_filename: Option<String>,
    sitemap_title: String,
    recent_first: bool,
    link_home: Option<String>,
    link_up: Option<String>,
    html_head: Option<String>,
    html_preamble: Option<String>,
    html_postamble: Option<String>,
    publish_action: PublishAction
}


/// A builder knows how to process a project
/// We have one builder for each PublishAction
pub trait Builder{
    fn build(project: Project) -> Result<()>;
}
