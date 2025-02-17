use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use gist::{download_gist, list_gists, Gists};
use tokio::task;
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
            folder,
            concurrent,
            limit,
        } => {
            info!("Fetching gists for user: {}", username);
            let gists: Gists = list_gists(&username, Some(limit)).await?;
            info!("Found {} gists", gists.len());

            // Create a vector to hold all download tasks
            let mut handles = vec![];

            // Create download tasks for each gist
            for gist in gists {
                let folder = folder.clone();
                let handle = tokio::spawn(async move {
                    match download_gist(&gist, &folder).await {
                        Ok(_) => info!("Successfully downloaded gist: {}", gist.id),
                        Err(e) => info!("Failed to download gist {}: {}", gist.id, e),
                    }
                });
                handles.push(handle);

                // If we've reached the concurrent limit, wait for one to complete
                if handles.len() >= concurrent {
                    if let Some(handle) = handles.pop() {
                        handle.await?;
                    }
                }
            }

            // Wait for remaining downloads to complete
            for handle in handles {
                handle.await?;
            }

            info!(
                "Download process completed. Check the '{}' directory",
                folder
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