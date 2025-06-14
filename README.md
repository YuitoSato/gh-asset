# gh-asset

A CLI tool to download GitHub user-attachments assets using GitHub CLI authentication.

## Features

- Download assets from GitHub issues and pull requests using asset ID
- Uses GitHub CLI (`gh`) for authentication
- Built with Rust for performance and reliability

## Prerequisites

- [GitHub CLI](https://cli.github.com/) installed and authenticated
- `curl` command available on your system

## Installation

### Download Pre-built Binary (Recommended)

Download the latest binary for your platform from [Releases](https://github.com/YuitoSato/gh-asset/releases/latest):

```bash
# macOS (Intel)
curl -L https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-x86_64-apple-darwin.tar.gz | tar -xz
sudo mv gh-asset /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-aarch64-apple-darwin.tar.gz | tar -xz
sudo mv gh-asset /usr/local/bin/

# Linux
curl -L https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo mv gh-asset /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/YuitoSato/gh-asset/releases/latest/download/gh-asset-x86_64-pc-windows-msvc.zip" -OutFile "gh-asset.zip"
Expand-Archive -Path "gh-asset.zip" -DestinationPath "."
```

### Building from Source

```bash
git clone https://github.com/YuitoSato/gh-asset.git
cd gh-asset
cargo build --release
```

The binary will be available at `target/release/gh-asset`.

## Usage

```bash
gh-asset download <asset_id> <destination_path>
```

### How to get Asset ID

When you upload files to GitHub issues or pull requests, GitHub creates URLs like:
```
https://github.com/user-attachments/assets/1234abcd-1234-1234-1234-1234abcd1234
```

The asset ID is the last part: `1234abcd-1234-1234-1234-1234abcd1234`

### Examples

```bash
# Download an asset using its asset ID
gh-asset download 1234abcd-1234-1234-1234-1234abcd1234 ./image.png

# Download another asset with a different filename
gh-asset download abcd1234-5678-9012-3456-789012345678 ./document.pdf
```

## Authentication

This tool requires GitHub CLI to be installed and authenticated. If you haven't authenticated yet:

```bash
gh auth login
```

## Error Handling

The tool will provide clear error messages for common issues:
- GitHub CLI not installed or not authenticated
- Invalid asset ID format
- Network errors during download
- File permission issues

## Testing

Run the test suite:

```bash
cargo test
```

## License

Apache-2.0 License - see LICENSE file for details.