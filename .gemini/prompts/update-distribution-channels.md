# Gemini Prompt: Update Distribution Channels (Homebrew & Scoop)

**Goal:** Automate the process of updating local repositories for `homebrew-tap` (macOS/Linux) and `scoop-bucket` (Windows) with a new release of this Rust CLI tool.

**Instructions for Gemini:**
When a user asks you to "update the distribution channels," "update homebrew and scoop," or "execute this file," follow these steps strictly:

---

### 1. Extract Metadata
Read `Cargo.toml` to extract:
- `name` (This is the `BINARY_NAME`, the command users install. The formula and manifest are named after it.)
- `version` (`VERSION`)
- `description` (`DESCRIPTION`)
- `repository` (Extract the `GITHUB_USER`, and the `REPO_NAME` from the last path segment. It may differ from `BINARY_NAME`, e.g. `liferay-workspace-updater` installs `lwu`.)

Then confirm the tag `v{{VERSION}}` exists (`git tag --list v{{VERSION}}`). If it does not, stop: the release must be tagged first (see the Releasing section of `README.md`). The Release workflow rejects tags that don't match `Cargo.toml`.

### 2. Prepare Template Values
- **Class Name:** Convert `BINARY_NAME` to PascalCase/CamelCase (e.g., `lfr-local` -> `LfrLocal`) for the `CLASS_NAME`.
- **Release URL:** `https://github.com/{{GITHUB_USER}}/{{REPO_NAME}}/archive/refs/tags/v{{VERSION}}.tar.gz`
- **Calculate SHA256:**
  *First, verify the release exists:*
  ```bash
  curl -Is <Release URL> | head -n 1
  ```
  If it returns 200, calculate the hash (extracting just the hex string):
  ```bash
  curl -sL <Release URL> | shasum -a 256 | cut -d ' ' -f 1
  ```
  This is your `{{SHA256}}`.

---

### 3. Update Homebrew (macOS/Linux)
The tap's `update-formulas.yml` workflow updates existing formulas from new GitHub releases every 2 hours, so this step is only needed when `Formula/{{BINARY_NAME}}.rb` does not exist yet. If it already exists, skip this step. For a new formula, also add an `Update {{BINARY_NAME}}` step to the tap's `.github/workflows/update-formulas.yml`:
```yaml
steps:
  # ...existing steps, before "Update README"...
  - name: Update {{BINARY_NAME}}
    run: ./scripts/update-formula.sh {{BINARY_NAME}} {{GITHUB_USER}}/{{REPO_NAME}}
```

- **Locate & Verify Tap:** Find the local path to the `homebrew-tap` repo (default: `../homebrew-tap`). Verify it is a git repository:
  ```bash
  [ -d "../homebrew-tap/.git" ] && echo "Tap found" || echo "Tap NOT found"
  ```
  *If not found, ask the user for the correct path before proceeding.*
- **Generate Formula:** Read `formula.rb.example`, replace placeholders, and write to `<homebrew-tap-path>/Formula/{{BINARY_NAME}}.rb`.
- **Commit & Push:**
  ```bash
  cd <homebrew-tap-path>
  git add Formula/{{BINARY_NAME}}.rb .github/workflows/update-formulas.yml
  git commit -m "feat: update {{BINARY_NAME}} v{{VERSION}}"
  git push origin main
  ```

---

### 4. Update Scoop (Windows)
The bucket's `auto-update.yml` workflow updates existing manifests from new GitHub releases, so this step is only needed when `bucket/{{BINARY_NAME}}.json` does not exist yet. If it already exists, skip this step.

- **Locate & Verify Bucket:** Find the local path to the `scoop-bucket` repo (default: `../scoop-bucket`). Verify it is a git repository:
  ```bash
  [ -d "../scoop-bucket/.git" ] && echo "Bucket found" || echo "Bucket NOT found"
  ```
  *If not found, ask the user for the correct path before proceeding.*
- **Generate Manifest:** Read `scoop.json.example`, replace placeholders, and write to `<scoop-bucket-path>/bucket/{{BINARY_NAME}}.json`.
- **Commit & Push:**
  ```bash
  cd <scoop-bucket-path>
  git add bucket/{{BINARY_NAME}}.json
  git commit -m "feat: update {{BINARY_NAME}} v{{VERSION}}"
  git push origin main
  ```

---

### 5. Keep the Templates
Do **not** delete `formula.rb.example` or `scoop.json.example`. They are needed for every future release.

---

**To the User:** 
To execute this, ask Gemini: *"Please execute the steps in .gemini/prompts/update-distribution-channels.md to update my distribution repositories located at ../homebrew-tap and ../scoop-bucket"*
