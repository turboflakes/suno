use crate::error::Error;
use bytes::Bytes;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::{debug, info};

const GITHUB_API: &'static str = "https://api.github.com";
const REPO_OWNER: &'static str = "turboflakes";
const REPO_NAME: &'static str = "suno";
const BIN_NAME: &'static str = "suno";

pub type Checksum = String;
pub type AssetName = String;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

impl Release {
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    message: String,
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

/// Starts the update process for the given version.
pub async fn start(client: &Client, version: Option<&str>) -> Result<Release, Error> {
    // Fetch release metadata
    let release = fetch_release(&client, version).await?;
    info!("✔︎ Found release {}", release.tag_name);

    // Check if already up to date
    let current = env!("CARGO_PKG_VERSION");
    if version.is_none() && release.tag_name.trim_start_matches('v') == current {
        return Err(Error::AlreadyUpToDate);
    }

    Ok(release)
}

pub async fn download(
    client: &Client,
    release: &Release,
    asset_name: &str,
) -> Result<(Bytes, Checksum), Error> {
    // Locate assets in release
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| Error::AssetNotFound(asset_name.to_string()))?;

    // Download checksum
    let checksum_name = format!("{}.sha256", asset_name);
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

    info!("✔︎ Assets downloaded");

    Ok((bytes, expected_hash))
}

/// Validates the checksum of the downloaded bytes against the expected hash.
pub fn validate(bytes: &Bytes, expected_hash: &str) -> Result<(), Error> {
    let actual_hash = hex::encode(Sha256::digest(bytes));
    if actual_hash != expected_hash {
        return Err(Error::InvalidChecksum);
    }

    info!("✔︎ Checksum verified");
    Ok(())
}

/// Extracts the binary from the downloaded bytes and replaces the existing binary.
pub fn extract_and_replace(bytes: &Bytes, asset_name: &str) -> Result<(), Error> {
    // Extract binary to temp location
    let (_tmp_dir, bin_path) = extract_binary(&bytes, &asset_name)?;
    debug!("Binary extracted to {}", bin_path.display());

    // Atomically replace current binary
    self_replace::self_replace(&bin_path)?;

    info!("✔︎ Binary replaced");
    Ok(())
}

/// Runs the update process for the given version.
pub async fn run_update(version: Option<&str>) -> Result<(), Error> {
    let client = Client::builder()
        .user_agent(format!("{}/{}", BIN_NAME, env!("CARGO_PKG_VERSION")))
        .build()?;

    // Start update process
    let release = start(&client, version).await?;

    // Resolve platform-specific asset name
    let asset_name = asset_name_for_platform()?;

    // Download binary archive
    let (bytes, expected_hash) = download(&client, &release, &asset_name).await?;

    // Validate SHA256
    let _ = validate(&bytes, &expected_hash)?;

    // Extract binary to temp location and replace current binary
    let _ = extract_and_replace(&bytes, &asset_name)?;

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
pub fn asset_name_for_platform() -> Result<String, Error> {
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
