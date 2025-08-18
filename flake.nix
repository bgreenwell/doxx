{
  description = "doxx - Expose the contents of .docx files without leaving your terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Use the latest stable Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Define the package
        doxx = pkgs.rustPlatform.buildRustPackage rec {
          pname = "doxx";
          version = "0.1.0";

          src = pkgs.fetchFromGitHub {
            owner = "bgreenwell";
            repo = "doxx";
            rev = "main"; # We will want to pin this to a specific commit/tag/revision
            sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # To be be updated
          };

          cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # To be updated

          # Native build inputs (build-time dependencies)
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];

          # Runtime dependencies based on Cargo.toml
          buildInputs = with pkgs; [
            # For crossterm terminal manipulation
            # For arboard clipboard functionality
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            
            # SSL/TLS support for potential future reqwest usage
            openssl
            
            # Standard build essentials
            stdenv.cc.cc.lib
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # macOS specific dependencies
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.CoreFoundation
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.AppKit # For clipboard support
            pkgs.darwin.apple_sdk.frameworks.Cocoa
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # Linux specific for clipboard
            xorg.libxcb
            wayland
            libxkbcommon
          ];

          # Environment variables
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          
          # Disable network access during build (standard for Nix)
          doCheck = false;

          meta = with pkgs.lib; {
            description = "Expose the contents of .docx files without leaving your terminal. Fast, safe, and smart — no Office required!";
            homepage = "https://github.com/bgreenwell/doxx";
            license = licenses.mit;
            maintainers = [ ]; # Add maintainer info if desired
            platforms = platforms.all;
          };
        };

      in
      {
        # Default package
        packages.default = doxx;
        packages.doxx = doxx;

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            
            # Development tools
            cargo-watch
            cargo-edit
            cargo-audit
            cargo-deny
            cargo-outdated
            cargo-expand # For macro expansion debugging
            
            # Build dependencies
            pkg-config
            openssl
            
            # Terminal and clipboard dependencies for development
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            
            # Additional development tools
            git
            just # task runner (optional)
            
            # LSP and formatting tools
            rust-analyzer
            
            # For testing .docx files and document creation
            pandoc
            libreoffice # for creating test .docx files
            
            # Debugging tools
            gdb
            valgrind
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # macOS specific
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.CoreFoundation
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.AppKit
            pkgs.darwin.apple_sdk.frameworks.Cocoa
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # Linux specific
            xorg.libxcb
            wayland
            libxkbcommon
          ];

          # Environment variables
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          
          # For better terminal support
          TERM = "xterm-256color";
          
          # Development shell hook
          shellHook = ''
            echo "❄️ Welcome to the doxx Nix development environment!"
            echo ""
            echo "📋 Dependencies loaded:"
            echo "  - Rust ${rustToolchain.version} with clippy, rustfmt, rust-src"
            echo "  - ratatui for terminal UI"
            echo "  - crossterm for cross-platform terminal"
            echo "  - arboard for clipboard support"
            echo "  - docx-rs for document parsing"
            echo ""
            echo "❄️ Nix commands:"
            echo "  nix build                - Build the project"
            echo "  nix run                  - Run doxx"
            echo "  nix run . -- --help      - Run with help flag"
            echo "  nix develop              - Enter this dev shell"
            echo "  nix flake check          - Run all checks (fmt, clippy, build)"
            echo ""
            echo "📄 Usage examples:"
            echo "  nix run . -- document.docx"
            echo "  nix run . -- document.docx --outline"
            echo "  nix run . -- document.docx --search 'keyword'"
            echo "  nix run . -- document.docx --export csv"
            echo ""
            echo "🔧 Development commands (if you need them):"
            echo "  cargo build              - Direct build (uses Nix env)"
            echo "  cargo watch -x run       - Live reload during development"
            echo "  cargo clippy             - Run linter"
            echo "  cargo fmt                - Format code"
            echo ""
            echo "💡 Pro tip: 'nix run github:bgreenwell/doxx -- file.docx' to run from anywhere!"
            echo ""
          '';
        };

        # Apps for easy running
        apps.default = {
          type = "app";
          program = "${doxx}/bin/doxx";
        };

        # Checks for CI
        checks = {
          build = doxx;
          
          # Add format check
          fmt-check = pkgs.runCommand "fmt-check" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${self}
            cargo fmt --all -- --check
            touch $out
          '';

          # Add clippy check
          clippy-check = pkgs.runCommand "clippy-check" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${self}
            cargo clippy --all-targets --all-features -- -D warnings
            touch $out
          '';
        };
      });
}
