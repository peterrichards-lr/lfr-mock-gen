## Liferay Mock Content Generator (Gemini Integration)

This repository includes a powerful CLI tool built in Rust that automatically generates realistic mock data for Liferay using the Gemini AI API. It creates 20 Basic Web Content articles, populates them with unique AI-generated titles, descriptions, and body paragraphs, and injects open-source placeholder images.

The tool uses Liferay's Headless Delivery REST APIs to push the content directly into your chosen Site, completely bypassing the need for complex `.lar` file imports or Groovy scripts.

### Prerequisites

1. **Rust and Cargo**: Ensure you have the latest stable version of Rust installed.
2. **Gemini API Key**: You will need a free API key from Google AI Studio.
3. **Liferay DXP/CE**: A running instance of Liferay 7.3+ (requires the Headless Delivery APIs).

### Installation & Build

Compile the application for release to ensure maximum performance:

```bash
cargo build --release
```

### Install via Homebrew (macOS / Linux)

```bash
brew tap peterrichards-lr/tap
brew install lfr-mock-gen
```

### Install via Scoop (Windows)

```powershell
scoop bucket add peterrichards-lr https://github.com/peterrichards-lr/scoop-bucket
scoop install lfr-mock-gen
```

Installing via Homebrew or Scoop builds from source, which avoids "Unidentified Developer" warnings on macOS and gives a user-level installation on Windows.

### Releasing

1. Bump `version` in `Cargo.toml`, run `cargo build` to update `Cargo.lock`, and commit.
2. Tag the commit with the matching version and push it: `git tag v1.2.3 && git push origin v1.2.3`. The Release workflow fails if the tag doesn't match `Cargo.toml`.
3. Update the Homebrew tap (and, for a first release, the Scoop bucket) from `formula.rb.example` and `scoop.json.example`. The Gemini prompt in `.gemini/prompts/update-distribution-channels.md` automates this:
   ```bash
   "Please execute the steps in .gemini/prompts/update-distribution-channels.md to update my distribution repositories"
   ```
   The Scoop bucket picks up later releases automatically.
