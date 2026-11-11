use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lfr-mock-gen")]
#[command(version, about = "Liferay Mock Content Generator via Gemini", long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: AppCommands,
}

#[derive(Subcommand)]
pub enum AppCommands {
    /// Generate mock Web Content with Gemini and create it in a Liferay site
    Data {
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

        #[arg(
            long,
            help = "Global site ID to search for the structure, if it cannot be resolved automatically"
        )]
        global_site_id: Option<u64>,

        #[arg(
            long,
            help = "The tone of the generated content (e.g., professional, casual, humorous)",
            default_value = "professional"
        )]
        tone: String,

        #[arg(
            long,
            help = "The purpose/type of the content (e.g., Blog, FAQ, News, Events)",
            default_value = "Blog"
        )]
        purpose: String,
    },
}
