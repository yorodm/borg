use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, error};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, create_dir_all},
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

pub mod attachment;
pub mod org;
pub mod sitemap;

const MAX_RECURSION: usize = 100;

#[derive(Debug, Serialize, Deserialize)]
pub enum PublishAction {
    #[serde(rename = "to-html")]
    ToHtml,
    #[serde(rename = "attachment")]
    Attachment,
}

impl Default for PublishAction {
    fn default() -> Self {
        Self::Attachment // By default we only copy
    }
}

/// A site config
/// Basically a list of `Project` definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct Site {
    pub projects: Vec<Project>,
}

impl Site {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut data = String::new();
        let mut file = File::open(&path)
            .context(format!(
                "Failed to open config file {}",
                path.as_ref().display()
            ))
            .map_err(|e| {
                error!("{}", e);
                e
            })?;
        file.read_to_string(&mut data)
            .context("Failed to read configuration")?;
        let cfg: Site = toml::from_str(&data)
            .context("Malformed configuration")
            .map_err(|e| {
                error!("{}", e);
                e
            })?;
        Ok(cfg)
    }
}

/// A publishing project.
/// Made this as close as possible to org-publish projects while still
/// considering my own use case
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
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
    fn from_project<P: AsRef<Path>>(project: &Project, root: P) -> Result<T>;
    fn build(&self) -> Result<()>;
}

// Helper to build globsets out of patterns
fn build_glob<S: AsRef<str>>(patterns: &[S]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for p in patterns.iter().map(|s| s.as_ref().trim()) {
        builder.add(Glob::new(p.as_ref()).context("Pattern is not valid")?);
    }
    let set = builder
        .build()
        .context("Failed to create a pattern set matcher")?;
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
    pub(crate) fn new_from_project<P: AsRef<Path>>(
        project: &crate::Project,
        root: P,
    ) -> Result<DirSettings> {
        debug!(
            "Processing {} with path {}",
            project.name,
            root.as_ref().display()
        );
        let include = match project.base_extension {
            Some(ref ext) => vec![ext.as_str()],
            None => vec!["*"],
        };
        let exclude = project
            .exclude
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        let entries = get_source_entries(
            &project.base_directory,
            &include,
            &exclude,
            project.recursive,
        )?;
        return Ok(DirSettings {
            files: entries,
            publish_dir: make_absolute_path(&project.publishing_directory, &root)?,
            source_dir: make_absolute_path(&project.base_directory, &root)?
                .canonicalize()
                .map_err(|e| {
                    error!("{}", e);
                    e
                })
                .context("Source directory doesn't exist or is it invalid")?,
        });
    }
}

pub(crate) fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    match path.as_ref().canonicalize() {
        Ok(p) => Ok(p),
        // we assume the directory does not exists and try to mkdir it
        Err(_) => {
            create_dir_all(&path)
                .map_err(|e| {
                    error!(
                        "Directory {} does not exist and cannot be created {}",
                        path.as_ref().display(),
                        e
                    );
                    std::io::Error::last_os_error()
                })
                .context("Failed creating directory tree")?;
            Ok(path.as_ref().to_owned())
        }
    }
}

fn make_absolute_path<P: AsRef<Path>, B: AsRef<Path>>(path: P, root: B) -> Result<PathBuf> {
    match RelativePath::from_path(&path) {
        Ok(p) => Ok(p.to_logical_path(root)),
        Err(e) => match e.kind() {
            relative_path::FromPathErrorKind::NonRelative => Ok(path.as_ref().into()),
            relative_path::FromPathErrorKind::NonUtf8 => bail!("{}", e),
            relative_path::FromPathErrorKind::BadSeparator => bail!("{}", e),
            _ => todo!(), // shouldn't
        },
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
    let include_entry = |path: &DirEntry| {
        // don't reject child directories
        path.file_type().is_dir() || include_patterns.is_match(path.path())
    };
    let exclude_patterns = build_glob(exclude_filter)?;
    let exclude_entry = |path: &DirEntry| !exclude_patterns.is_match(path.path());
    let recursion_level = if recursive { MAX_RECURSION } else { 1 };
    let walker = WalkDir::new(&path)
        .max_depth(recursion_level)
        .into_iter()
        .filter_entry(|p| !is_hidden(p) && exclude_entry(p) && include_entry(p));
    let entries = walker
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<DirEntry>, _>>()
        .context(format!(
            "Error reading {} directory content",
            &path.as_ref().display()
        ))
        .map_err(|e| {
            error!("{}", e);
            e
        })?
        .into_iter()
        //The walker will return the directories as entries
        .filter(|p| p.file_type().is_file())
        .collect();
    return Ok(entries);
}
