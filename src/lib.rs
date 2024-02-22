use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub mod attachment;
pub mod org;

#[derive(Debug, Serialize, Deserialize)]
pub enum PublishAction {
    ToHtml,
    Attachment,
    Rss,
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
    projects: Vec<Project>,
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
    recursive: usize,
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
    publish_action: PublishAction,
}

/// A builder knows how to process a project
/// We have one builder for each PublishAction
pub trait Builder: Sized {
    fn from_project(project: &Project) -> Result<Self>;
    fn build(&self) -> Result<()>;
}

// Helper to build globsets out of patterns
fn build_glob<S: AsRef<str>>(patterns: &[S]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns.iter() {
        builder.add(Glob::new(p.as_ref())?);
    }
    let set = builder.build()?;
    Ok(set)
}

// Helper to skip hidden files
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

/// Get all the entries in a directory, applying filters defined in the project
pub(crate) fn get_source_entries<P, S>(
    path: P,
    include_filter: &[S],
    exclude_filter: &[S], // unused by now
    recursion_level: usize,
) -> Result<Vec<DirEntry>, anyhow::Error>
where
    P: AsRef<Path>,
    S: AsRef<str>,
{
    let include_patterns = build_glob(include_filter)?;
    let include_entry = |path: &DirEntry| include_patterns.is_match(path.path());
    let exclude_patterns = build_glob(exclude_filter)?;
    let exclude_entry = |path: &DirEntry| !exclude_patterns.is_match(path.path());
    let walker = WalkDir::new(path)
        .into_iter()
        .filter_entry(|p| !is_hidden(p) && include_entry(p) && !exclude_entry(p));
    let entries = walker.filter_map(Result::ok).collect();
    return Ok(entries);
}
