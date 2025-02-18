# Downloading Github Gists

This Rust application provides functionality for downloading GitHub Gists through a command-line interface. It allows users to either list gists for a specified GitHub username or download them to a local folder. 
The main functionality is split into two commands: 

- "list": displays gists for a given user with an optional limit on the number of gists to show, 
- "download": fetches gists to a local directory with configurable concurrency. 

## Usage

- List gists for a user

```bash
local_gist list --username <username> --limit 10
```

- Download gists

```bash
local_gist download --username <username> --limit 10 --concurrency 10
```

The download command uses a semaphore to control concurrent downloads, using O(n) memory for tracking the gists are are going to be downloaded. Could implement streaming downloads to disk during pagination to reduce memory usage. All operations are handled asynchronously.

## Dependencies

The application uses the clap crate for command-line argument parsing, tokio for asynchronous operations, and tracing for logging, while handling errors with anyhow and thiserror. Probably should be using only one of those.

```toml
anyhow = { version = "1.0" }
clap = { version = "4.5.30", features = ["derive"] }
reqwest = { version = "0.12.12", features = ["json", "native-tls-vendored"] }
serde = { version = "1.0.217", features = ["derive"] }
serde_json = { version = "1.0.138" }
tokio = { version = "1.43.0", features = ["full"] }
tracing = { version = "0.1.41" }
tracing-subscriber = { version = "0.3.19" }
thiserror = { version = "2.0.11" }
```


## Checking the performance of conccurent and non-conccurent runs:

```bash 
🦀 v1.83.0 ➜ time ./target/release/local_gist download --username github_user --limit 10 --concurrency 1
2025-02-17T19:02:34.004202Z  INFO local_gist: Fetching gists for user: github_user
2025-02-17T19:02:34.009201Z  INFO local_gist::gist: Limit: Some(10), per page: 10
2025-02-17T19:02:34.009224Z  INFO local_gist::gist: Requesting URL: https://api.github.com/users/github_user/gists?per_page=10&page=1
2025-02-17T19:02:34.235131Z  INFO local_gist::gist: Status: 200 OK
2025-02-17T19:02:34.235182Z  INFO local_gist::gist: Wait, there is more!
2025-02-17T19:02:34.235192Z  INFO local_gist::gist: rate_limit: Some("60") rate_remaining: Some("51")
2025-02-17T19:02:34.273598Z  INFO local_gist: Found 10 gists
2025-02-17T19:02:34.491504Z  INFO local_gist: Successfully downloaded gist: b7a4c8e2f15d93602481d7c9a4f35e90
2025-02-17T19:02:34.980733Z  INFO local_gist: Successfully downloaded gist: d139e5f87b2c46a0835d94c21b67f4e8
2025-02-17T19:02:35.239863Z  INFO local_gist: Successfully downloaded gist: 7f2e9d4c83b51a6074c9e238f5d1a9b0
2025-02-17T19:02:35.470465Z  INFO local_gist: Successfully downloaded gist: c8f3e7d619b542a078fc93d1e5b4a826
2025-02-17T19:02:35.693965Z  INFO local_gist: Successfully downloaded gist: 9d5b2e8a4f16c730951d84b2e7c3f9a5
2025-02-17T19:02:35.876966Z  INFO local_gist: Successfully downloaded gist: e4f8c2b759a31d604872e9f5c1b3a6d0
2025-02-17T19:02:36.073260Z  INFO local_gist: Successfully downloaded gist: 1b5d9c7e4a382f6054d8b9c3f7e2a150
2025-02-17T19:02:36.282677Z  INFO local_gist: Successfully downloaded gist: 8a3f6d2c5b917e4083c2d5f9a4b16e70
2025-02-17T19:02:36.497782Z  INFO local_gist: Successfully downloaded gist: 5e2b8f4d9c713a6042f8e5d1b7c9a360
2025-02-17T19:02:36.729214Z  INFO local_gist: Successfully downloaded gist: 2d7f4e8b5c916a3074f2d8e9c5b1a730
2025-02-17T19:02:36.729661Z  INFO local_gist: Download complete: 15 files downloaded to /Users/github_user/code/home/experiments/local-gist/gists

________________________________________________________
Executed in    2.77 secs      fish           external
   usr time   92.44 millis    0.21 millis   92.23 millis
   sys time   76.20 millis    3.44 millis   72.76 millis
```


```bash
🦀 v1.83.0 ➜ time ./target/release/local_gist download --username github_user --limit 10 --concurrency 10
2025-02-17T19:02:39.571974Z  INFO local_gist: Fetching gists for user: github_user
2025-02-17T19:02:39.577177Z  INFO local_gist::gist: Limit: Some(10), per page: 10
2025-02-17T19:02:39.577199Z  INFO local_gist::gist: Requesting URL: https://api.github.com/users/github_user/gists?per_page=10&page=1
2025-02-17T19:02:39.804499Z  INFO local_gist::gist: Status: 200 OK
2025-02-17T19:02:39.804609Z  INFO local_gist::gist: Wait, there is more!
2025-02-17T19:02:39.804621Z  INFO local_gist::gist: rate_limit: Some("60") rate_remaining: Some("50")
2025-02-17T19:02:39.848739Z  INFO local_gist: Found 10 gists
2025-02-17T19:02:40.036340Z  INFO local_gist: Successfully downloaded gist: 1b5d9c7e4a382f6054d8b9c3f7e2a150
2025-02-17T19:02:40.069865Z  INFO local_gist: Successfully downloaded gist: 8a3f6d2c5b917e4083c2d5f9a4b16e70
2025-02-17T19:02:40.069967Z  INFO local_gist: Successfully downloaded gist: 9d5b2e8a4f16c730951d84b2e7c3f9a5
2025-02-17T19:02:40.074411Z  INFO local_gist: Successfully downloaded gist: 2d7f4e8b5c916a3074f2d8e9c5b1a730
2025-02-17T19:02:40.078445Z  INFO local_gist: Successfully downloaded gist: b7a4c8e2f15d93602481d7c9a4f35e90
2025-02-17T19:02:40.086307Z  INFO local_gist: Successfully downloaded gist: d139e5f87b2c46a0835d94c21b67f4e8
2025-02-17T19:02:40.097590Z  INFO local_gist: Successfully downloaded gist: 5e2b8f4d9c713a6042f8e5d1b7c9a360
2025-02-17T19:02:40.099293Z  INFO local_gist: Successfully downloaded gist: e4f8c2b759a31d604872e9f5c1b3a6d0
2025-02-17T19:02:40.129145Z  INFO local_gist: Successfully downloaded gist: 7f2e9d4c83b51a6074c9e238f5d1a9b0
2025-02-17T19:02:40.230134Z  INFO local_gist: Successfully downloaded gist: c8f3e7d619b542a078fc93d1e5b4a826
2025-02-17T19:02:40.230701Z  INFO local_gist: Download complete: 15 files downloaded to /Users/github_user/code/home/experiments/local-gist/gists

________________________________________________________
Executed in  698.07 millis    fish           external
   usr time   62.48 millis    0.18 millis   62.30 millis
   sys time   60.73 millis    4.67 millis   56.06 millis

```