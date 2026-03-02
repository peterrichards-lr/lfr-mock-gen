use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lfr-mock-gen")]
#[command(about = "Liferay Mock Content Generator via Gemini", long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: AppCommands,
}

#[derive(Subcommand)]
pub enum AppCommands {
    Env {
        #[arg(short, long)]
        target: Option<String>,
    },
    Data {
        #[arg(long, help = "Force operation")]
        force: bool,

        #[arg(long, help = "Environment variable name containing the Gemini API key")]
        api_env: String,

        #[arg(long, help = "Liferay Site ID (groupId)")]
        group_id: u64,

        #[arg(long, help = "Liferay Host URL (e.g., http://localhost:8080)")]
        liferay_url: String,

        #[arg(long, help = "Liferay Basic Auth username (e.g., test@liferay.com)")]
        liferay_user: String,

        #[arg(long, help = "Liferay Basic Auth password")]
        liferay_pass: String,

        #[arg(long, help = "Liferay Content Structure ID or Name")]
        structure_id: String,
    },
}
