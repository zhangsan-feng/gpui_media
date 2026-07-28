mod command;
mod environment;
mod support;
mod verify;
mod workflow;

use anyhow::Result;
use command::{Action, Options};

fn main() {
    if let Err(error) = run() {
        eprintln!("Windows build failed: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = Options::parse()?;
    match options.action {
        Action::Doctor => workflow::doctor(&options),
        Action::Package => workflow::package(&options),
        Action::Support => workflow::support(&options),
        Action::Verify => workflow::verify(&options),
    }
}
