# Liferay Rust Tool Template

A template for building high-performance, cross-platform CLI tools for Liferay DXP development and operations.

## Features

- **Cross-Platform:** Pre-configured GitHub Actions to build for Windows, Linux, and macOS (ARM/Intel).
- **Liferay Aware:** Standardized logic for path resolution and `portal-ext.properties` parsing.
- **Modern CLI:** Built on `clap` for a professional command-line experience.

## Project Structure

```plaintext
.
├── .github/workflows/release.yml # Multi-OS CI/CD
├── src/
│   ├── main.rs          # Command routing
│   ├── core/
│   │   ├── mod.rs       # Core traits (Interchangeable)
│   │   └── env.rs       # Local Filesystem Adapter (Optional)
│   ├── utils/
│   │   ├── mod.rs       # Utility re-exports
│   │   └── xml.rs       # Recursive XML logic (Tomcat/Docker Configs)
│   └── cli.rs           # Multi-tool command definitions
├── .gitignore
├── Cargo.toml
└── LICENSE (MIT)
```

## Getting Started

1. Click **"Use this template"** on GitHub.
2. Update the `name` and `description` in `Cargo.toml`.
3. Customize the subcommands in `src/main.rs`.
4. Push a tag (e.g., `v1.0.0`) to trigger the automated release.

## Development

```bash
# Build locally
cargo build

# Run with arguments
cargo run -- --help
