use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use gist::{download_gist, list_gists, Gists};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
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
        } => handle_download(username, folder, concurrency, limit).await?,
        Commands::List { username, limit } => {
            info!("Listing the first {:?} gists for user: {}", limit, username);
            let gists: Gists = list_gists(&username, limit).await?;
            for gist in gists {
                info!("{}", gist);
            }
        }
    }
    Ok(())
}

async fn handle_download(
    username: String,
    folder: String,
    concurrency: usize,
    limit: Option<u32>,
) -> Result<()> {
    info!("Fetching gists for user: {username}");
    let gists: Vec<gist::Gist> = list_gists(&username, limit).await?;
    let number_of_files: &usize = &gists.iter().map(|g| g.files.len()).sum::<usize>();
    info!("Found {} gists", gists.len());

    let abs_path = PathBuf::from(&folder)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&folder));

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut set = JoinSet::new();

    for gist in gists {
        let sem = Arc::clone(&semaphore);
        let folder = folder.clone();
        set.spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            if let Err(e) = download_gist(&gist, &folder).await {
                info!("Failed to download gist {}: {}", gist.id, e);
                return;
            }
            info!("Successfully downloaded gist: {}", gist.id);
        });
    }

    while let Some(res) = set.join_next().await {
        res?;
    }

    info!(
        "Download complete: {} files downloaded to {}",
        number_of_files,
        abs_path.display()
    );

    Ok(())
}
