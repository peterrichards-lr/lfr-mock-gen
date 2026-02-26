mod cli;
mod core;
mod utils;

use crate::cli::{App, AppCommands};
use crate::core::{LiferayWorkspace, Workspace};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    let ws = LiferayWorkspace {
        current_dir: std::env::current_dir().unwrap_or_default(),
    };

    match args.command {
        AppCommands::Env { target } => {
            // .map_err converts String to anyhow::Error
            let root = ws.find_root().map_err(anyhow::Error::msg)?;
            println!("Environment check for {:?} in root: {:?}", target, root);

            // This line uses utils to remove the "unused import" warning
            let _ = utils::find_elements_by_name;
        }
        AppCommands::Data { force } => {
            println!("Data operation initiated (Force={})", force);
        }
    }

    Ok(())
}
