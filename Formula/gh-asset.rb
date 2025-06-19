class GhAsset < Formula
  desc "CLI tool to download GitHub issue/PR assets with smart file extension detection"
  homepage "https://github.com/YuitoSato/gh-asset"
  url "https://github.com/YuitoSato/gh-asset/archive/v0.1.6.tar.gz"
  sha256 "c94898360ae81d0b16c6e9c206a8531e9836d844db51098115648da3f4fff08d"
  license "Apache-2.0"
  version "0.1.6"

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