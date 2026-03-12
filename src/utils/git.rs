use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub fn git_add(root: &Path, files: &[&Path]) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("add")
        .args(files)
        .current_dir(root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git add failed")
    }
}

#[allow(dead_code)]
pub fn git_commit(root: &Path, message: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .current_dir(root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git commit failed")
    }
}

#[allow(dead_code)]
pub fn git_push(root: &Path) -> anyhow::Result<()> {
    let status = Command::new("git").arg("push").current_dir(root).status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git push failed")
    }
}

#[allow(dead_code)]
pub fn is_git_dirty(root: &Path) -> bool {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(root)
        .output();

    match output {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}
