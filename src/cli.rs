use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lfr-tool")]
#[command(about = "Generic Liferay Rust Tool", long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: AppCommands,
}

#[derive(Subcommand)]
pub enum AppCommands {
    /// Generic environment command
    Env {
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Generic data command
    Data {
        #[arg(long)]
        force: bool,
    },
}
