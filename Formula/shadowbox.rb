class Shadowbox < Formula
  desc "Zero-trust copy-on-write sandbox and interactive time-machine for AI coding agents"
  homepage "https://github.com/Nari-Ab/shadowbox"
  version "0.1.0"
  license "Apache-2.0"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/Nari-Ab/shadowbox/releases/download/v0.1.0/shadowbox-macos-arm64.tar.gz"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/Nari-Ab/shadowbox/releases/download/v0.1.0/shadowbox-macos-x86_64.tar.gz"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/Nari-Ab/shadowbox/releases/download/v0.1.0/shadowbox-linux-x86_64.tar.gz"
  end

  def install
    bin.install "shadowbox"
  end

  test do
    assert_match "shadowbox 0.1.0", shell_output("#{bin}/shadowbox --version")
  end
end
