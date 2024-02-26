use anyhow::{Context, Result};
use borg::attachment::AttachmentsHandler;
use borg::org::PublishHandler;
use borg::{Builder, Project, Config};
use seahorse::{App, Context as AppContext, ActionResult, Command, ActionError};
use std::env;


fn run_project_builder(project: &Project) -> Result<()> {
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

fn action(ctx: &AppContext) -> ActionResult{
    match Config::from_file(&ctx.args[1]) {
        Ok(config) => {
            let result : Result<()> = config.projects.iter()
                .map(|p| run_project_builder(p))
                .collect();
            result.map_err(|_| -> ActionError {
                ActionError{
                    message: "Error running publish".to_owned()
                }})
        },
        Err(_) => Err(ActionError {
            message: "Error reading configuration".to_owned()
        }),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let build = Command::new("publish")
        .description("Execute the publish actions on the projects")
        .usage("borg_cli publish [path to config]")
        .action_with_result(action);
    let app = App::new("borg_cli")
        .author(env!("CARGO_PKG_AUTHORS"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .usage("borg_cli [command] [arg]")
        .version(env!("CARGO_PKG_VERSION"))
        .command(build);
    Ok(app.run(args))
}
