use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::process::Command;
use url::Url;

#[derive(Parser)]
#[command(name = "gh-asset")]
#[command(about = "A CLI tool to download GitHub issue/PR assets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Download {
        #[arg(help = "GitHub asset URL or path")]
        source: String,
        #[arg(help = "Destination file path")]
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

    fn download(&self, source: &str, destination: &str) -> Result<()> {
        let url = self.parse_asset_url(source)?;
        self.download_with_curl(&url, destination)
    }

    fn parse_asset_url(&self, source: &str) -> Result<String> {
        if source.starts_with("http://") || source.starts_with("https://") {
            let parsed_url = Url::parse(source)
                .map_err(|e| anyhow!("Invalid URL format: {}", e))?;
            
            if !parsed_url.host_str().unwrap_or("").contains("github") {
                return Err(anyhow!("URL must be from GitHub"));
            }
            
            Ok(source.to_string())
        } else {
            Err(anyhow!("Source must be a valid GitHub URL"))
        }
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
        Commands::Download { source, destination } => {
            let downloader = AssetDownloader::new()?;
            downloader.download(&source, &destination)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asset_url_valid_github_url() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        let result = downloader.parse_asset_url("https://github.com/user/repo/issues/1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_asset_url_invalid_url() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        let result = downloader.parse_asset_url("not_a_url");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_asset_url_non_github_url() {
        let auth = GitHubAuth { token: "fake_token".to_string() };
        let downloader = AssetDownloader { auth };
        
        let result = downloader.parse_asset_url("https://example.com/file.jpg");
        assert!(result.is_err());
    }
}