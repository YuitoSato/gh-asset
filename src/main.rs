use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "gh-asset")]
#[command(about = "A CLI tool to download GitHub issue/PR assets using GitHub CLI authentication")]
#[command(long_about = "Download assets from GitHub issues and pull requests using GitHub CLI authentication.

PREREQUISITES:
  • GitHub CLI (gh) installed and authenticated
  • curl command available on your system

INSTALLATION:
  Download the latest binary from: https://github.com/YuitoSato/gh-asset/releases/latest

  # macOS (Intel)
  curl -L https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-x86_64-apple-darwin.tar.gz | tar -xz
  sudo mv gh-asset /usr/local/bin/

  # macOS (Apple Silicon)
  curl -L https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-aarch64-apple-darwin.tar.gz | tar -xz
  sudo mv gh-asset /usr/local/bin/

  # Linux
  curl -L https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-x86_64-unknown-linux-gnu.tar.gz | tar -xz
  sudo mv gh-asset /usr/local/bin/

AUTHENTICATION:
  If you haven't authenticated GitHub CLI yet:
  gh auth login

EXAMPLES:
  # Download an asset using asset ID
  gh-asset download 1234abcd-1234-1234-1234-1234abcd1234 ./image.png

  # Download another asset with a different filename
  gh-asset download abcd1234-5678-9012-3456-789012345678 ./document.pdf")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download assets from GitHub using asset ID
    Download {
        #[arg(help = "GitHub asset ID (e.g., 1234abcd-1234-1234-1234-1234abcd1234)")]
        asset_id: String,
        #[arg(help = "Local file path where the asset will be saved")]
        destination: String,
    },
}

struct GitHubAuth {
    token: String,
}

impl GitHubAuth {
    fn new() -> Result<Self> {
        let output = Command::new("gh")
            .args(["auth", "token"])
            .output()
            .map_err(|e| anyhow!("Failed to execute gh command: {}. Make sure GitHub CLI is installed and authenticated.", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("GitHub CLI authentication failed: {}", error_msg));
        }

        let token = String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("Failed to parse gh auth token output: {}", e))?
            .trim()
            .to_string();

        if token.is_empty() {
            return Err(anyhow!("GitHub CLI token is empty. Please run 'gh auth login' first."));
        }

        Ok(GitHubAuth { token })
    }

    fn get_token(&self) -> &str {
        &self.token
    }
}

struct AssetDownloader {
    auth: GitHubAuth,
}

impl AssetDownloader {
    fn new() -> Result<Self> {
        let auth = GitHubAuth::new()?;
        Ok(AssetDownloader { auth })
    }

    fn download(&self, asset_id: &str, destination: &str) -> Result<()> {
        let url = self.build_asset_url(asset_id)?;
        self.download_with_curl(&url, destination)
    }

    fn build_asset_url(&self, asset_id: &str) -> Result<String> {
        // Validate asset ID format (UUID-like with hyphens)
        if !self.is_valid_asset_id(asset_id) {
            return Err(anyhow!("Invalid asset ID format. Expected format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"));
        }
        
        Ok(format!("https://github.com/user-attachments/assets/{}", asset_id))
    }
    
    fn is_valid_asset_id(&self, asset_id: &str) -> bool {
        // Check if asset_id contains only alphanumeric characters, hyphens, and has reasonable length
        if asset_id.is_empty() || asset_id.len() < 20 || asset_id.len() > 50 {
            return false;
        }
        
        // Check if it contains only valid characters (alphanumeric and hyphens)
        // Must contain at least one hyphen (typical UUID format)
        asset_id.chars().all(|c| c.is_alphanumeric() || c == '-') && asset_id.contains('-')
    }

    fn download_with_curl(&self, url: &str, destination: &str) -> Result<()> {
        println!("Downloading {} to {}", url, destination);

        let output = Command::new("curl")
            .args([
                "-L",
                "-H", &format!("Authorization: token {}", self.auth.get_token()),
                "-H", "Accept: application/vnd.github.v3+json",
                "-o", destination,
                url,
            ])
            .output()
            .map_err(|e| anyhow!("Failed to execute curl command: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("curl command failed: {}", error_msg));
        }

        println!("Successfully downloaded to {}", destination);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Download { asset_id, destination } => {
            let downloader = AssetDownloader::new()?;
            downloader.download(&asset_id, &destination)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_asset_id_valid() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        assert!(downloader.is_valid_asset_id("1234abcd-1234-1234-1234-1234abcd1234"));
        assert!(downloader.is_valid_asset_id("abcd1234-5678-9012-3456-789012345678"));
    }

    #[test]
    fn test_is_valid_asset_id_invalid() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        assert!(!downloader.is_valid_asset_id(""));
        assert!(!downloader.is_valid_asset_id("abc"));
        assert!(!downloader.is_valid_asset_id("invalid-id"));
        assert!(!downloader.is_valid_asset_id("invalid@id"));
        assert!(!downloader.is_valid_asset_id("id with spaces"));
        assert!(!downloader.is_valid_asset_id("a1b2c3d4e5")); // No hyphen
    }

    #[test]
    fn test_build_asset_url() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        let result = downloader.build_asset_url("1234abcd-1234-1234-1234-1234abcd1234");
        assert_eq!(result.unwrap(), "https://github.com/user-attachments/assets/1234abcd-1234-1234-1234-1234abcd1234");
    }

    #[test]
    fn test_build_asset_url_invalid() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        let result = downloader.build_asset_url("invalid@id");
        assert!(result.is_err());
    }
}