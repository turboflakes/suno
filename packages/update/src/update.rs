use crate::error::Error;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::{debug, info};

const GITHUB_API: &'static str = "https://api.github.com";
const REPO_OWNER: &'static str = "turboflakes";
const REPO_NAME: &'static str = "suno";
const BIN_NAME: &'static str = "suno";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Checks if a new version is available for download.
pub async fn check_for_update() -> Result<String, Error> {
    let client = Client::builder()
        .user_agent(format!("{}/{}", BIN_NAME, env!("CARGO_PKG_VERSION")))
        .build()?;
    let release = fetch_release(&client, None).await?;

    let current = env!("CARGO_PKG_VERSION");
    if release.tag_name.trim_start_matches('v') != current {
        return Ok(release.tag_name);
    }

    Err(Error::NewVersionNotFound)
}

/// Runs the update process for the given version.
pub async fn run(version: Option<&str>) -> Result<(), Error> {
    let client = Client::builder()
        .user_agent(format!("{}/{}", BIN_NAME, env!("CARGO_PKG_VERSION")))
        .build()?;

    // Fetch release metadata
    let release = fetch_release(&client, version).await?;
    info!("✔︎ Found release: {}", release.tag_name);

    // Check if already up to date
    let current = env!("CARGO_PKG_VERSION");
    if version.is_none() && release.tag_name.trim_start_matches('v') == current {
        info!("— Already up to date (v{})", current);
        return Ok(());
    }

    // Resolve platform-specific asset name
    let asset_name = asset_name_for_platform()?;
    let checksum_name = format!("{}.sha256", asset_name);

    // Locate assets in release
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| Error::AssetNotFound(asset_name.to_string()))?;

    let checksum_asset = release
        .assets
        .iter()
        .find(|a| a.name == checksum_name)
        .ok_or_else(|| Error::ChecksumNotFound(checksum_name))?;

    // Download checksum
    let checksum_text = client
        .get(&checksum_asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Support both bare hash and `sha256sum` format ("hash  filename")
    let expected_hash = checksum_text
        .split_whitespace()
        .next()
        .ok_or_else(|| Error::InvalidChecksumFormat)?
        .to_lowercase();

    // Download binary archive
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    // Validate SHA256
    let actual_hash = hex::encode(Sha256::digest(&bytes));
    if actual_hash != expected_hash {
        return Err(Error::InvalidChecksum);
    }
    info!("✔︎ Checksum verified");

    // Extract binary to temp location
    let (_tmp_dir, bin_path) = extract_binary(&bytes, &asset_name)?;

    debug!("Extracted binary to {}", bin_path.display());

    // Atomically replace current binary
    self_replace::self_replace(&bin_path)?;
    info!("✔︎ Updated to {}", release.tag_name);

    Ok(())
}

/// Fetches the release information from the GitHub API.
async fn fetch_release(client: &Client, version: Option<&str>) -> Result<Release, Error> {
    let url = match version {
        Some(v) => format!(
            "{}/repos/{}/{}/releases/tags/v{}",
            GITHUB_API, REPO_OWNER, REPO_NAME, v
        ),
        None => format!(
            "{}/repos/{}/{}/releases/latest",
            GITHUB_API, REPO_OWNER, REPO_NAME
        ),
    };
    debug!("Fetching release from {}", url);

    let res = client.get(&url).send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let release = res.json::<Release>().await?;
            Ok(release)
        }
        _ => {
            let response = res.json::<ErrorResponse>().await?;
            Err(Error::Other(response.message))
        }
    }
}

/// Returns the asset name for the current platform.
fn asset_name_for_platform() -> Result<String, Error> {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => return Err(Error::UnsupportedArchitecture(other.to_string())),
    };

    let (vendor_os, ext) = match std::env::consts::OS {
        "linux" => ("unknown-linux-gnu", "tar.gz"),
        "macos" => ("apple-darwin", "tar.gz"),
        other => return Err(Error::UnsupportedOs(other.to_string())),
    };

    Ok(format!("{}-{}-{}.{}", BIN_NAME, arch, vendor_os, ext))
}

/// Extracts the binary from a `.tar.gz` archive into a temp dir.
/// Returns the `TempDir` handle (keeps the dir alive) and the path to the binary.
fn extract_binary(bytes: &[u8], asset_name: &str) -> Result<(tempfile::TempDir, PathBuf), Error> {
    let tmp_dir = tempfile::tempdir()?;
    let out_path = tmp_dir.path().join(BIN_NAME);

    if asset_name.ends_with(".tar.gz") {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let gz = GzDecoder::new(bytes);
        let mut archive = Archive::new(gz);

        let mut found = false;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            if path.file_name().and_then(|n| n.to_str()) == Some(BIN_NAME) {
                entry.unpack(&out_path)?;
                found = true;
                break;
            }
        }
        if !found {
            return Err(Error::BinaryNotFound);
        }
    } else {
        return Err(Error::UnknownFormat(asset_name.to_string()));
    }

    // Ensure executable bit on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok((tmp_dir, out_path))
}
