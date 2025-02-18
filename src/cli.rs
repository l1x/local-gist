use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "local-gist")]
#[command(author = "Gist Downloader CLI")]
#[command(version = "1.0")]
#[command(about = "Downloads GitHub Gists", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Download gists for a specific user
    Download {
        /// GitHub username
        #[arg(short, long)]
        username: String,

        /// Directory to save gists
        #[arg(short, long, default_value = "gists")]
        folder: String,

        /// Number of concurrency downloads
        #[arg(short, long, default_value_t = 4)]
        concurrency: usize,

        /// Maximum number of gists to download
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// List gists for a specific user
    List {
        /// GitHub username
        #[arg(short, long)]
        username: String,

        /// Maximum number of gists to list
        #[arg(short, long)]
        limit: Option<u32>,
    },
}
