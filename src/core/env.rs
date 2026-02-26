use std::fs;
use std::path::{Path, PathBuf};

pub trait Workspace {
    fn find_root(&self) -> Result<PathBuf, String>;

    #[allow(dead_code)]
    fn find_tomcat(&self, root: &Path) -> Result<PathBuf, String>;
}

pub struct LiferayWorkspace {
    pub current_dir: PathBuf,
}

impl Workspace for LiferayWorkspace {
    fn find_root(&self) -> Result<PathBuf, String> {
        // Only check the current directory for the 'bundles' folder
        if self.current_dir.join("bundles").exists() {
            Ok(self.current_dir.clone())
        } else {
            Err(
                "Liferay Workspace not found in the current directory. (Missing 'bundles' folder)"
                    .to_string(),
            )
        }
    }

    fn find_tomcat(&self, root: &Path) -> Result<PathBuf, String> {
        let bundles = root.join("bundles");
        let entries = fs::read_dir(bundles).map_err(|_| "Cannot read 'bundles' folder")?;

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if (name.starts_with("tomcat-") || name == "tomcat") && entry.path().is_dir() {
                return Ok(entry.path());
            }
        }
        Err("Tomcat directory not found inside the 'bundles' folder.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_tomcat_flexible_naming() {
        let dir = tempdir().unwrap();
        let bundles_path = dir.path().join("bundles");
        let tomcat_path = bundles_path.join("tomcat-9.0.90");

        fs::create_dir_all(&tomcat_path).unwrap();

        let ws = LiferayWorkspace {
            current_dir: dir.path().to_path_buf(),
        };

        // This simulates finding the root and then the tomcat dir
        let found_tomcat = ws.find_tomcat(dir.path()).unwrap();
        assert!(found_tomcat.to_string_lossy().contains("tomcat-9.0.90"));
    }
}
