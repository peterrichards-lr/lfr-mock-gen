# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_Nothing yet._

## [1.0.3] - 2026-11-11

### Changed

- Upgrade `jsonschema` to 0.60, pinning validation to Draft 7 (the previous implicit default).
- Label Dependabot pull requests `no-issue-needed`, alongside the default `dependencies` and ecosystem labels, so bot PRs are marked as intentionally having no linked issue.
- Update Rust dependencies:
  - `tempfile` 3.26.0 → 3.27.0
  - `serde` 1.0.228 → 1.0.229
  - `base64` 0.21.7 → 0.23.1
  - `reqwest` 0.11.27 → 0.13.5
  - `chrono` 0.4.44 → 0.4.45
- Update GitHub Actions:
  - `actions/upload-artifact` 7.0.1 → 7.0.2
  - `actions/download-artifact` 8.0.1 → 8.0.2
  - `dtolnay/rust-toolchain` (pinned commit updated)

## [1.0.2] - 2026-11-11

### Added

- Add `CHANGELOG.md`, with sections backfilled for earlier releases.
- Add `scripts/release.py` (creates annotated release tags and extracts release notes from `CHANGELOG.md`) and `scripts/doctor.py` (checks for unresolved template placeholders).
- Add the shared `.githooks/pre-commit` hook, `.cargo/config.toml` aliases (`setup-hooks`, `lint`, `fmt-check`), and `.github/dependabot.yml` for weekly GitHub Actions and Cargo updates.
- Add an Intel macOS (`x86_64-apple-darwin`) binary to the release build.

### Changed

- Replace `.github/workflows/release.yml` with the template v0.7.0 workflow: it derives binary and asset names from `Cargo.toml`, requires an annotated tag matching the `Cargo.toml` version, and uses the matching `CHANGELOG.md` section as the release notes.
- Pin GitHub Actions in `.github/workflows/rust.yml` to commit SHAs.
- Update `.gemini/prompts/update-distribution-channels.md` to the template version, which names the formula and manifest after the binary and keeps `formula.rb.example` and `scoop.json.example`.
- Rewrite `README.md` with installation, usage for the `data` and `env` commands, and release instructions.

### Fixed

- Correct the README's description of what the tool generates: each run creates 5 entries for a chosen content structure (not 20 Basic Web Content articles), and image fields get `picsum.photos` URLs from Gemini rather than uploaded images.

## [1.0.1] - 2026-11-11

### Added

- Add a `--version` flag. Earlier binaries didn't expose `--version` and declared version 0.1.0 regardless of the release tag.
- Add `formula.rb.example`, and `repository` and `license` to `Cargo.toml`.
- Add Homebrew, Scoop and Releasing sections to `README.md`.

### Changed

- Align the `Cargo.toml` version with the release tags.
- Fail the release workflow when the tag doesn't match the `Cargo.toml` version.
- Align `scoop.json.example` with the Scoop bucket's existing manifests.
- Bump `openssl` from 0.10.75 to 0.10.82 to resolve security advisories.

### Fixed

- Close an unclosed code block in `README.md` that rendered the rest of the file as code.

## [1.0.0] - 2026-03-13

### Changed

- Align with template v0.2.0: `LiferayProject` workspace discovery, shared Git utilities, and `anyhow` error handling.
- Refactor the release workflow to avoid race conditions between build jobs.

### Fixed

- Fix Clippy dead-code warnings by using the `Workspace` methods in the `env` command.

## [0.1.1] - 2026-03-12

### Added

- Generate mock Web Content with Gemini from any site content structure, looked up by ID or name.
- Validate Gemini's output against a JSON Schema derived from the structure before creating content.
- Map Text, Rich Text, Numeric, Date, Boolean, Image, Color, Select and Multiselect fields.
- Add `--tone` and `--purpose` options to the `data` command.
- Add the `env` command for Liferay project discovery.
- Add the Rust CI and release workflows, and track `Cargo.lock`.

<!-- markdownlint-disable MD049 -->
---
*Last Updated: 2026-11-11* | *Last Reviewed: 2026-11-11*
