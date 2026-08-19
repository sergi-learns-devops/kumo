class Kumo < Formula
  desc "Cloud instance type recommender for GCP, AWS & Azure"
  homepage "https://github.com/sergiserra/kumo"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/sergiserra/kumo/releases/download/v#{version}/kumo-macos-aarch64.tar.gz"
      # sha256 "PLACEHOLDER"
    else
      url "https://github.com/sergiserra/kumo/releases/download/v#{version}/kumo-macos-x86_64.tar.gz"
      # sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/sergiserra/kumo/releases/download/v#{version}/kumo-linux-aarch64.tar.gz"
      # sha256 "PLACEHOLDER"
    else
      url "https://github.com/sergiserra/kumo/releases/download/v#{version}/kumo-linux-x86_64.tar.gz"
      # sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "kumo"
  end

  test do
    assert_match "kumo", shell_output("#{bin}/kumo --version")
  end
end
