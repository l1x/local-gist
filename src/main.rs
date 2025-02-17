use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use gist::{download_gist, list_gists, Gists};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, Level};
use tracing_subscriber;

mod cli;
mod gist;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Download {
            username,
            folder,
            concurrency,
            limit,
        } => {
            info!("Fetching gists for user: {}", username);
            let gists: Gists = list_gists(&username, Some(limit)).await?;
            info!("Found {} gists", gists.len());

            let abs_path = PathBuf::from(&folder)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(&folder));
            let total_files = gists.iter().map(|g| g.files.len()).sum::<usize>();

            // Create a semaphore to limit concurrency downloads
            let semaphore = Arc::new(Semaphore::new(concurrency));
            let mut handles = vec![];

            for gist in gists {
                let folder: String = folder.clone();
                let sem: Arc<Semaphore> = Arc::clone(&semaphore);

                let handle = tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let _permit = sem.acquire().await.unwrap();
                    // The permit is automatically released when _permit is dropped

                    match download_gist(&gist, &folder).await {
                        Ok(_) => info!("Successfully downloaded gist: {}", gist.id),
                        Err(e) => info!("Failed to download gist {}: {}", gist.id, e),
                    }
                });
                handles.push(handle);
            }

            // Wait for all downloads to complete
            for handle in handles {
                handle.await?;
            }

            info!(
                "Download complete: {} files downloaded to {}",
                total_files,
                abs_path.display()
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
