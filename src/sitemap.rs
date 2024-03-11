use std::io::Write;

use chrono::{Local, DateTime};
use orgize::Org;
use rss::{Channel, ChannelBuilder, Item};
use anyhow::Result;
use crate::Project;

/// Sitemap generator:
/// This will construct the `index.org` file that will be used as a sitemap
#[derive(Default)]
pub struct SitemapGenerator<'a> {
    index_site: Org<'a>,
}

impl<'a> SitemapGenerator<'a> {
    pub fn new(project: &Project) -> Self {
        SitemapGenerator {
            index_site: Org::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RssGenerator {
    channel: Channel,
    items: Vec<Item>
}

impl RssGenerator {

    pub fn add_article(&self, title: &str, description: Option<&str>, date: DateTime<Local>) {
        todo!()
    }

    pub fn new(self, project: &Project) -> Self {
        let mut builder = ChannelBuilder::default();
        builder.title(&project.name);
        if let Some(ref home) = project.link_home {
            builder.link(home);
        };
        if let Some(ref description) = project.description {
            builder.description(description);
        };
        RssGenerator {
            channel: builder.build(),
            items: Vec::new()
        }
    }

    // Consume the generator
    pub fn generate<W: Write>(mut self, w:W) -> Result<()>{
        self.channel.set_items(self.items);
        self.channel.write_to(w)?;
        Ok(())
    }
}
