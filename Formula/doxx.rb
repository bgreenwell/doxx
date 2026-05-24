class Doxx < Formula
  desc "Terminal document viewer for .docx files"
  homepage "https://github.com/bgreenwell/doxx"
  version "0.1.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/bgreenwell/doxx/releases/download/v#{version}/doxx-aarch64-apple-darwin.tar.xz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/bgreenwell/doxx/releases/download/v#{version}/doxx-x86_64-apple-darwin.tar.xz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bgreenwell/doxx/releases/download/v#{version}/doxx-aarch64-unknown-linux-gnu.tar.xz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/bgreenwell/doxx/releases/download/v#{version}/doxx-x86_64-unknown-linux-gnu.tar.xz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "doxx"
  end

  test do
    system "#{bin}/doxx", "--version"
  end
end
