class GhAsset < Formula
  desc "CLI tool to download GitHub issue/PR assets"
  homepage "https://github.com/YuitoSato/gh-asset"
  url "https://github.com/YuitoSato/gh-asset/archive/v0.1.0.tar.gz"
  sha256 "YOUR_SHA256_HERE"
  license "MIT"

  depends_on "rust" => :build
  depends_on "gh"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Test that the binary exists and shows help
    assert_match "A CLI tool to download GitHub issue/PR assets", shell_output("#{bin}/gh-asset --help")
  end
end