class GhAsset < Formula
  desc "CLI tool to download GitHub issue/PR assets with smart file extension detection"
  homepage "https://github.com/YuitoSato/gh-asset"
  url "https://github.com/YuitoSato/gh-asset/archive/v0.1.5.tar.gz"
  sha256 "44bd7c5bee93e7ca1c9a0bc87f7f471e1c840a2bf40c2e317180f0e7b4d1772b"
  license "Apache-2.0"
  version "0.1.5"

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