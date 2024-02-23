use anyhow::{anyhow, Result, Context};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::{path::{Path, PathBuf}, str::FromStr, fs::File, io::Read};
use walkdir::{DirEntry, WalkDir};

pub mod attachment;
pub mod org;

#[derive(Debug, Serialize, Deserialize)]
pub enum PublishAction {
    #[serde(rename="to-html")]
    ToHtml,
    #[serde(rename="attachment")]
    Attachment,
    #[serde(rename="rss")]
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut data = String::new();
        let mut file = File::open(&path)
            .context(format!("Failed to open config file {}", path.as_ref().to_string_lossy()))?;
        file.read_to_string(&mut data)
            .context("Failed to read configuration")?;
        let cfg :Config = toml::from_str(&data)
            .context("Malformed configuration")?;
        Ok(cfg)
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

#[derive(Debug)]
pub(crate) struct DirSettings {
    files: Vec<DirEntry>,
    source_dir: PathBuf, // We need this for stripping
    publish_dir: PathBuf,
}

impl DirSettings {
    pub(crate) fn try_from_roject(project: &crate::Project) -> Result<DirSettings> {
        if let Some(ref path) = project.base_directory {
            let include = match project.base_directory {
                Some(ref path) => vec![path.as_str()],
                None => vec!["*"],
            };
            let exclude: Vec<&str> = project.exclude.iter().map(|s| s.as_str()).collect();
            let entries = get_source_entries(path, &include, &exclude, project.recursive)?;
            return Ok(DirSettings {
                files: entries,
                publish_dir: PathBuf::from_str(&project.publishing_directory)?,
                source_dir: PathBuf::from_str(path.as_str())?,
            });
        }
        Err(anyhow!(
            "Base directory does not exists or hasn't been defined"
        ))
    }
}

impl TryFrom<&Project> for DirSettings {
    type Error = anyhow::Error;

    fn try_from(value: &Project) -> std::result::Result<Self, Self::Error> {
        DirSettings::try_from_roject(value)
    }
}

/// Get all the entries in a directory, applying filters defined in the project
pub(crate) fn get_source_entries<P, S>(
    path: P,
    include_filter: &[S],
    exclude_filter: &[S], // unused by now
    recursive: bool,
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
        .filter_entry(|p| {
            !is_hidden(p) && include_entry(p) && !exclude_entry(p) && (p.file_type().is_dir() == recursive)
        });
    let entries: Result<_,_> = walker.into_iter().into_iter().collect();
    return Ok(entries.context("Failed to read items from base directory")?)
}
