use anyhow::{Context, Result};
use borg::attachment::AttachmentsHandler;
use borg::org::PublishHandler;
use borg::{Builder, Project};
use orgize::elements::ExportBlock;
use seahorse::App;
use std::env;

fn run_project_builder(project: Project) -> Result<()> {
    match project.publish_action {
        borg::PublishAction::ToHtml => {
            PublishHandler::from_project(&project)
                .context(format!(
                    "Cannot create html publisher from {}",
                    project.name
                ))?
                .build()
                .context("Failed publishing html project")
        }
        borg::PublishAction::Attachment => {
            AttachmentsHandler::from_project(&project)
                .context(format!(
                    "Cannot create attachment handler from {}",
                    project.name
                ))?
                .build()
                .context("Failed publishing attachments project")
        }
        borg::PublishAction::Rss => todo!(),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let app = App::new("borg_cli")
        .author(env!("CARGO_PKG_AUTHORS"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .usage("borg_cli [command] [arg]")
        .version(env!("CARGO_PKG_VERSION"));
    Ok(app.run(args))
}
