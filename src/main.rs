use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
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

    async fn download(&self, asset_id: &str, destination: &str) -> Result<()> {
        let url = self.build_asset_url(asset_id)?;
        let safe_destination = self.validate_destination_path(destination)?;
        self.download_with_reqwest(&url, &safe_destination).await
    }

    fn build_asset_url(&self, asset_id: &str) -> Result<String> {
        // Validate asset ID format (UUID-like with hyphens)
        if !self.is_valid_asset_id(asset_id) {
            return Err(anyhow!("Invalid asset ID format. Expected format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"));
        }
        
        Ok(format!("https://github.com/user-attachments/assets/{}", asset_id))
    }
    
    fn is_valid_asset_id(&self, asset_id: &str) -> bool {
        // Asset ID must be at least 20 characters and at most 50 characters
        if asset_id.len() < 20 || asset_id.len() > 50 {
            return false;
        }
        
        // Must contain at least one hyphen
        if !asset_id.contains('-') {
            return false;
        }
        
        // GitHub asset IDs follow a specific UUID-like pattern
        // Example: 1234abcd-1234-1234-1234-1234abcd1234
        if let Ok(re) = Regex::new(r"^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$") {
            if re.is_match(asset_id) {
                return true;
            }
        }
        
        // Also allow GitHub's actual format which can include alphanumeric + specific chars
        // Must be longer than simple pattern and contain hyphens in specific positions
        if let Ok(github_re) = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9\-]{18,48}[a-zA-Z0-9]$") {
            if github_re.is_match(asset_id) && asset_id.matches('-').count() >= 2 {
                return true;
            }
        }
        
        false
    }
    
    fn validate_destination_path(&self, destination: &str) -> Result<PathBuf> {
        let path = Path::new(destination);
        
        // Check for path traversal attempts
        if destination.contains("..") {
            return Err(anyhow!("Path traversal detected in destination path"));
        }
        
        // Ensure the path doesn't start with absolute paths to system directories
        if path.is_absolute() {
            let path_str = path.to_string_lossy();
            if path_str.starts_with("/etc") || 
               path_str.starts_with("/usr") || 
               path_str.starts_with("/var") || 
               path_str.starts_with("/sys") || 
               path_str.starts_with("/proc") ||
               path_str.starts_with("/root") ||
               path_str.starts_with("/boot") {
                return Err(anyhow!("Access to system directories is not allowed"));
            }
        }
        
        // Canonicalize the path to resolve any remaining traversal attempts
        let current_dir = std::env::current_dir()
            .map_err(|e| anyhow!("Failed to get current directory: {}", e))?;
        
        let resolved_path = if path.is_relative() {
            current_dir.join(path)
        } else {
            path.to_path_buf()
        };
        
        // Ensure the resolved path is within or below the current directory for relative paths
        if path.is_relative() {
            match resolved_path.canonicalize() {
                Ok(canonical) => {
                    if !canonical.starts_with(&current_dir) {
                        return Err(anyhow!("Destination path must be within current directory"));
                    }
                }
                Err(_) => {
                    // Path doesn't exist yet, check parent directory
                    if let Some(parent) = resolved_path.parent() {
                        if parent.exists() {
                            match parent.canonicalize() {
                                Ok(canonical_parent) => {
                                    if !canonical_parent.starts_with(&current_dir) {
                                        return Err(anyhow!("Destination path must be within current directory"));
                                    }
                                }
                                Err(e) => return Err(anyhow!("Failed to validate destination path: {}", e)),
                            }
                        }
                    }
                }
            }
        }
        
        // Check filename for invalid characters
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.contains('\0') || filename_str.trim().is_empty() {
                return Err(anyhow!("Invalid filename"));
            }
        }
        
        Ok(resolved_path)
    }

    async fn download_with_reqwest(&self, url: &str, destination: &PathBuf) -> Result<()> {
        println!("Downloading {} to {}", url, destination.display());

        // Create a secure HTTP client with proper TLS verification
        let client = reqwest::Client::builder()
            .user_agent("gh-asset/0.1.3")
            .timeout(std::time::Duration::from_secs(300)) // 5 minutes timeout
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        // Make the request with authorization header
        let response = client
            .get(url)
            .header("Authorization", format!("token {}", self.auth.get_token()))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send HTTP request: {}", e))?;

        // Check response status
        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP request failed with status: {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown error")
            ));
        }

        // Get the response bytes
        let bytes = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        // Create parent directories if they don't exist
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create parent directories: {}", e))?;
        }

        // Write to file securely
        let mut file = File::create(destination)
            .map_err(|e| anyhow!("Failed to create destination file: {}", e))?;
        
        file.write_all(&bytes)
            .map_err(|e| anyhow!("Failed to write to destination file: {}", e))?;
        
        file.sync_all()
            .map_err(|e| anyhow!("Failed to sync file to disk: {}", e))?;

        println!("Successfully downloaded to {}", destination.display());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Download { asset_id, destination } => {
            let downloader = AssetDownloader::new()?;
            downloader.download(&asset_id, &destination).await?;
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
        
        // Valid UUID format
        assert!(downloader.is_valid_asset_id("1234abcd-1234-1234-1234-1234abcd1234"));
        // Valid GitHub format (more than 20 chars with multiple hyphens)
        assert!(downloader.is_valid_asset_id("a1b2c3d4e5f6g7h8i9j0-k1l2m3n4-o5p6q7r8"));
    }

    #[test]
    fn test_is_valid_asset_id_invalid() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        assert!(!downloader.is_valid_asset_id(""));
        assert!(!downloader.is_valid_asset_id("abc"));
        assert!(!downloader.is_valid_asset_id("invalid@id"));
        assert!(!downloader.is_valid_asset_id("id with spaces"));
        assert!(!downloader.is_valid_asset_id("a1b2c3d4e5")); // No hyphen
        assert!(!downloader.is_valid_asset_id("../../../etc/passwd"));
        assert!(!downloader.is_valid_asset_id("'; rm -rf /; '"));
    }

    #[test]
    fn test_validate_destination_path_safe() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        // Safe relative paths
        assert!(downloader.validate_destination_path("test.png").is_ok());
        assert!(downloader.validate_destination_path("./test.png").is_ok());
        assert!(downloader.validate_destination_path("subdir/test.png").is_ok());
    }

    #[test]
    fn test_validate_destination_path_unsafe() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        // Path traversal attempts
        assert!(downloader.validate_destination_path("../test.png").is_err());
        assert!(downloader.validate_destination_path("../../etc/passwd").is_err());
        assert!(downloader.validate_destination_path("/etc/passwd").is_err());
        assert!(downloader.validate_destination_path("/usr/bin/evil").is_err());
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
        
        let result = downloader.build_asset_url("../../../etc/passwd");
        assert!(result.is_err());
    }
}