use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use gist::{download_gist, list_gists, Gists};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::{runtime::Handle, sync::Semaphore};
use tracing::{debug, error, info, Level};
use tracing_subscriber;

mod cli;
mod gist;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .with_thread_ids(true)
        .with_thread_names(true)
        .with_max_level(Level::INFO)
        .init();

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

    let abs_path = PathBuf::from(&folder)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&folder));

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut download_set = JoinSet::new();
    let mut monitor_set = JoinSet::new();

    monitor_set.spawn(async move { monitor_tasks().await });

    info!("Found {} gists", gists.len());

    for gist in gists {
        let sem = Arc::clone(&semaphore);
        let folder = folder.clone();

        download_set.spawn(async move {
            let _permit = sem.acquire().await;

            if let Err(e) = download_gist(&gist, &folder).await {
                error!("Failed to download gist {}: {}", gist.id, e);
                return;
            }

            info!("Successfully downloaded gist: {}", gist.id);
        });
    }

    info!("All the tasks have been created");

    // Waits until one of the tasks in the set completes and returns its output.
    // Returns None if the set is empty.

    while let Some(res) = download_set.join_next().await {
        res?
    }

    monitor_set.abort_all();

    info!(
        "Download complete: {} files downloaded to {}",
        number_of_files,
        abs_path.display()
    );

    Ok(())
}

async fn monitor_tasks() {
    let handle = Handle::current();
    loop {
        let metrics = handle.metrics();
        debug!("Number of workers: {}", metrics.num_workers());
        debug!("Number of alive tasks: {}", metrics.num_alive_tasks());
        debug!("Global queue depth: {}", metrics.global_queue_depth());
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
