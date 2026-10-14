# Contributing to ShadowBox

Thank you for contributing to ShadowBox.

## Development Setup

Requirements:
- Rust 1.75+
- macOS (recommended for APFS CoW testing) or Linux

Clone and run tests:

```bash
git clone https://github.com/Nari-Ab/shadowbox.git
cd shadowbox
cargo test --all
```

## Pull Requests

1. Fork the repository and create your branch from `main`.
2. Ensure all tests pass (`cargo test --all`).
3. Ensure formatting and clippy pass (`cargo fmt --check` and `cargo clippy`).
4. Keep commit messages concise and descriptive.
5. Open a pull request describing the changes and problem solved.
