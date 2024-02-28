use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use walkdir::{DirEntry, WalkDir};

pub mod attachment;
pub mod org;
const MAX_RECURSION: usize = 100;

#[derive(Debug, Serialize, Deserialize)]
pub enum PublishAction {
    #[serde(rename = "to-html")]
    ToHtml,
    #[serde(rename = "attachment")]
    Attachment,
    #[serde(rename = "rss")]
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
    pub projects: Vec<Project>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut data = String::new();
        let mut file = File::open(&path).context(format!(
            "Failed to open config file {}",
            path.as_ref().to_string_lossy()
        ))?;
        file.read_to_string(&mut data)
            .context("Failed to read configuration")?;
        let cfg: Config = toml::from_str(&data).context("Malformed configuration")?;
        Ok(cfg)
    }
}

/// A publishing project.
/// Made this as close as possible to org-publish projects while still
/// considering my own use case
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Project {
    pub name: String,
    pub base_directory: String,
    pub base_extension: Option<String>,
    #[serde(default)]
    pub recursive: bool,
    pub publishing_directory: String,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub auto_sitemap: bool,
    pub sitemap_filename: Option<String>,
    pub sitemap_title: Option<String>,
    #[serde(default)]
    pub recent_first: bool,
    pub link_home: Option<String>,
    pub link_up: Option<String>,
    pub html_head: Option<String>,
    pub html_preamble: Option<String>,
    pub html_postamble: Option<String>,
    #[serde(default)]
    pub publish_action: PublishAction,
}

/// A builder knows how to process a project
/// We have one builder for each PublishAction
pub trait Builder<T> {
    fn from_project(project: &Project) -> Result<T>;
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

#[derive(Debug, Default)]
pub(crate) struct DirSettings {
    files: Vec<DirEntry>,
    source_dir: PathBuf, // We need this for stripping
    publish_dir: PathBuf,
}

impl DirSettings {
    pub(crate) fn try_from_project(project: &crate::Project) -> Result<DirSettings> {
        let include = match project.base_extension {
            Some(ref ext) => vec![ext.as_str()],
            None => vec!["*"],
        };
        let exclude: Vec<&str> = project.exclude.iter().map(|s| s.as_str()).collect();
        let entries = get_source_entries(
            &project.base_directory,
            &include,
            &exclude,
            project.recursive,
        )?;
        dbg!("Etries are {:?}", &entries);
        return Ok(DirSettings {
            files: entries,
            publish_dir: PathBuf::from_str(&project.publishing_directory)?,
            source_dir: PathBuf::from_str(&project.base_directory)?
                .canonicalize()
                .context("Source directory doesn't exist or is it invalid")?,
        });
    }
}

impl TryFrom<&Project> for DirSettings {
    type Error = anyhow::Error;

    fn try_from(value: &Project) -> std::result::Result<Self, Self::Error> {
        DirSettings::try_from_project(value)
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
    let recursion_level = if recursive { MAX_RECURSION } else { 1 };
    let walker = WalkDir::new(&path)
        .max_depth(recursion_level)
        .into_iter()
        .filter_entry(|p| !is_hidden(p) && include_entry(p) && exclude_entry(p));
    let entries = walker
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<DirEntry>, _>>()
        .context(format!("Error reading {} directory content", &path.as_ref().to_string_lossy()))?;
    return Ok(entries);
}
