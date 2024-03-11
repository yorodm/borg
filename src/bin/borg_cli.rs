use anyhow::{Context, Result};
use borg::attachment::AttachmentsBuilder;
use borg::org::PublishBuilder;
use borg::{Builder, Site, Project};
use log::{error, info};
use seahorse::{ActionError, ActionResult, App, Command, Context as AppContext};
use std::env;
use std::path::Path;
use std::process::exit;

fn run_project_builder(project: &Project, root: &Path) -> Result<()> {
    info!("Processing project {}", project.name);
    match project.publish_action {
        borg::PublishAction::ToHtml => PublishBuilder::from_project(&project, root)
            .context(format!(
                "Cannot create html publisher from {}",
                project.name
            ))?
            .build(),
        borg::PublishAction::Attachment => AttachmentsBuilder::from_project(&project, root)
            .context(format!(
                "Cannot create attachment handler from {}",
                project.name
            ))?
            .build()
            .map_err(|e| {
                error!("{}", e);
                e
            })
            .context("Failed publishing attachments project"),
    }
}

fn action(ctx: &AppContext) -> ActionResult {
    let Some(config_file) = ctx.args.get(0) else {
        error!("No config file provided");
        exit(1);
    };
    let config_file_path = Path::new(&config_file).canonicalize().map_err(|e| {
        error!("{}", e);
        ActionError {
            message: e.to_string(),
        }
    })?;
    // if we made it here we exist and we have a parent
    let root = config_file_path.parent().unwrap();
    match Site::from_file(&config_file_path) {
        Ok(config) => {
            let result = config
                .projects
                .iter()
                .map(|p| run_project_builder(p, &root))
                .collect::<Result<_>>();
            result.map_err(|e| -> ActionError {
                ActionError {
                    message: format!("Error running publish {}", e),
                }
            })
        }
        Err(e) => Err(ActionError {
            message: format!("Error reading configuration {}", e),
        }),
    }
}

fn main() -> Result<()> {
    env_logger::init();
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
