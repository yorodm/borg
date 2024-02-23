use std::io::Result;
use std::env;
use seahorse::App;

fn main() -> Result<()> {
    let args : Vec<String> = env::args().collect();
    let app = App::new("borg_cli")
        .author(env!("CARGO_PKG_AUTHORS"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .usage("borg_cli [command] [arg]")
        .version(env!("CARGO_PKG_VERSION"));
    Ok(app.run(args))
}
