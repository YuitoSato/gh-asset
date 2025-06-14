# gh-asset

A CLI tool to download GitHub issue/PR assets using GitHub CLI authentication.

## Features

- Download assets from GitHub issues and pull requests
- Uses GitHub CLI (`gh`) for authentication
- Built with Rust for performance and reliability

## Prerequisites

- [GitHub CLI](https://cli.github.com/) installed and authenticated
- `curl` command available on your system

## Installation

### Using Homebrew (Recommended)

```bash
brew tap YuitoSato/gh-asset
brew install gh-asset
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
gh-asset download <source_url> <destination_path>
```

### Examples

```bash
# Download an image from a GitHub issue
gh-asset download https://github.com/user/repo/assets/123456/image.png ./image.png

# Download an attachment from a PR
gh-asset download https://github.com/user/repo/assets/789012/document.pdf ./document.pdf
```

## Authentication

This tool requires GitHub CLI to be installed and authenticated. If you haven't authenticated yet:

```bash
gh auth login
```

## Error Handling

The tool will provide clear error messages for common issues:
- GitHub CLI not installed or not authenticated
- Invalid URLs (non-GitHub URLs)
- Network errors during download
- File permission issues

## Testing

Run the test suite:

```bash
cargo test
```

## License

Apache-2.0 License - see LICENSE file for details.