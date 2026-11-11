# Gemini Prompt: Update Distribution Channels (Homebrew & Scoop)

**Goal:** Automate the process of updating local repositories for `homebrew-tap` (macOS/Linux) and `scoop-bucket` (Windows) with a new release of this Rust CLI tool.

**Instructions for Gemini:**
When a user asks you to "update the distribution channels," "update homebrew and scoop," or "execute this file," follow these steps strictly:

---

### 1. Extract Metadata
Read `Cargo.toml` to extract:
- `name` (This is the `REPO_NAME` and `BINARY_NAME`)
- `version` (`VERSION`)
- `description` (`DESCRIPTION`)
- `repository` (Extract the `GITHUB_USER` and ensure `REPO_NAME` matches)

Then confirm the tag `v{{VERSION}}` exists (`git tag --list v{{VERSION}}`). If it does not, stop: the `Cargo.toml` version must be bumped and tagged first (the Release workflow rejects tags that don't match `Cargo.toml`).

### 2. Prepare Template Values
- **Class Name:** Convert `REPO_NAME` to PascalCase/CamelCase (e.g., `lfr-local` -> `LfrLocal`) for the `CLASS_NAME`.
- **Release URL:** `https://github.com/{{GITHUB_USER}}/{{REPO_NAME}}/archive/refs/tags/v{{VERSION}}.tar.gz`
- **Calculate SHA256:**
  *Wait for user confirmation that the release is published.* Then run:
  ```bash
  curl -sL <Release URL> | shasum -a 256
  ```
  This is your `{{SHA256}}`.

---

### 3. Update Homebrew (macOS/Linux)
- **Locate Tap:** Find the local path to the `homebrew-tap` repo (default: `../homebrew-tap`).
- **Generate Formula:** Read `formula.rb.example`, replace placeholders, and write to `<homebrew-tap-path>/Formula/{{REPO_NAME}}.rb`.
- **Commit & Push:**
  ```bash
  cd <homebrew-tap-path>
  git add Formula/{{REPO_NAME}}.rb
  git commit -m "feat: update {{REPO_NAME}} v{{VERSION}}"
  git push origin main
  ```

---

### 4. Update Scoop (Windows)
The bucket's `auto-update.yml` workflow updates existing manifests from new GitHub releases, so this step is only needed when `bucket/{{REPO_NAME}}.json` does not exist yet. If it already exists, skip this step.

- **Locate Bucket:** Find the local path to the `scoop-bucket` repo (default: `../scoop-bucket`).
- **Generate Manifest:** Read `scoop.json.example`, replace placeholders, and write to `<scoop-bucket-path>/bucket/{{REPO_NAME}}.json`.
- **Commit & Push:**
  ```bash
  cd <scoop-bucket-path>
  git add bucket/{{REPO_NAME}}.json
  git commit -m "feat: update {{REPO_NAME}} v{{VERSION}}"
  git push origin main
  ```

---

**To the User:** 
To execute this, ask Gemini: *"Please execute the steps in .gemini/prompts/update-distribution-channels.md to update my distribution repositories located at ../homebrew-tap and ../scoop-bucket"*
