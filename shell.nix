{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "discourse-tui";

  buildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rustfmt
    clippy

    # Build dependencies
    pkg-config
    openssl
  ];

  shellHook = ''
    echo "Discourse TUI - Terminal UI for Discourse"
    echo "=========================================="
    echo ""
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
    echo "Commands:"
    echo "  cargo run              - Run the application"
    echo "  cargo build --release  - Build optimized binary"
    echo "  cargo test             - Run tests"
    echo "  cargo clippy           - Lint code"
    echo ""
  '';
}
