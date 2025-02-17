use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use gist::{list_gists, Gists};
use tracing::{info, Level};
use tracing_subscriber;

mod cli;
mod gist;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Download {
            username,
            output,
            concurrent,
            limit,
        } => {
            info!("Fetching gists for user: {}", username);
            let gists: Gists = list_gists(&username, Some(limit)).await?;
            info!("Found {} gists", gists.len());
            info!(
                "Download process completed. Check the '{}' directory",
                output
            );
        }
        Commands::List { username, limit } => {
            info!("Listing gists for user: {}", username);
            let gists: Gists = list_gists(&username, Some(limit)).await?;
            for gist in gists {
                info!("{}", gist);
            }
        }
    }
    Ok(())
}
