use anyhow::{Context, Result, anyhow};
use ignore::gitignore::GitignoreBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Get the XDG config home directory, respecting XDG_CONFIG_HOME env var
pub fn get_xdg_config_home() -> Result<PathBuf> {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg_config))
    } else {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".config"))
    }
}

/// Load ignore rules from .stowignore file if it exists
pub fn load_ignore_rules(package_source: &Path) -> Result<Option<ignore::gitignore::Gitignore>> {
    let ignore_file = package_source.join(".stowignore");
    if ignore_file.exists() {
        let mut builder = GitignoreBuilder::new(package_source);
        builder.add(&ignore_file);
        Ok(Some(builder.build()?))
    } else {
        Ok(None)
    }
}

/// Stow a package by creating symlinks from source to target
pub fn stow_package(
    source: &Path,
    target: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
) -> Result<()> {
    // Create target directory if it doesn't exist
    if !target.exists() {
        fs::create_dir_all(target).context("Failed to create target directory")?;
    }

    // Walk through source directory, filtering out ignored entries
    let walker = WalkDir::new(source).follow_links(false).into_iter();

    for entry in walker.filter_entry(|e| {
        // Get relative path
        let path = e.path();
        let relative_path = match path.strip_prefix(source) {
            Ok(p) => p,
            Err(_) => return true, // Keep entry if we can't get relative path
        };

        // Always include root
        if relative_path.as_os_str().is_empty() {
            return true;
        }

        // Check if ignored
        if let Some(gi) = gitignore {
            let matched = gi.matched(relative_path, e.file_type().is_dir());
            if matched.is_ignore() {
                println!("  Ignoring: {}", relative_path.display());
                return false; // Skip this entry and its descendants
            }
        }

        true // Include entry
    }) {
        let entry = entry?;
        let path = entry.path();

        // Get relative path from source
        let relative_path = path
            .strip_prefix(source)
            .context("Failed to get relative path")?;

        // Skip the root directory itself
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        // Skip .stowignore file
        if relative_path.file_name() == Some(std::ffi::OsStr::new(".stowignore")) {
            continue;
        }

        let target_path = target.join(relative_path);

        if path.is_dir() {
            // Create directory if it doesn't exist
            if !target_path.exists() {
                fs::create_dir_all(&target_path).context(format!(
                    "Failed to create directory: {}",
                    target_path.display()
                ))?;
                println!("  Created directory: {}", relative_path.display());
            }
        } else {
            // Create parent directory if needed
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            // Check if symlink already exists
            if target_path.exists() || target_path.is_symlink() {
                // Check if it's already pointing to the correct location
                if target_path.is_symlink() {
                    let existing_link = fs::read_link(&target_path)?;
                    if existing_link == path {
                        println!("  Already linked: {}", relative_path.display());
                        continue;
                    }
                }

                return Err(anyhow!(
                    "Target already exists: {}. Please remove it manually or use --rm first.",
                    target_path.display()
                ));
            }

            // Create symlink
            #[cfg(unix)]
            std::os::unix::fs::symlink(path, &target_path).context(format!(
                "Failed to create symlink: {}",
                target_path.display()
            ))?;

            #[cfg(windows)]
            {
                if path.is_dir() {
                    std::os::windows::fs::symlink_dir(path, &target_path)?;
                } else {
                    std::os::windows::fs::symlink_file(path, &target_path)?;
                }
            }

            println!("  Linked: {}", relative_path.display());
        }
    }

    Ok(())
}

/// Remove a stowed package by deleting symlinks that point to source
pub fn remove_package(
    source: &Path,
    target: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
) -> Result<()> {
    if !target.exists() {
        return Err(anyhow!(
            "Target package does not exist: {}",
            target.display()
        ));
    }

    // Walk through source directory to find what should be removed
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry?;
        let path = entry.path();

        // Get relative path from source
        let relative_path = path
            .strip_prefix(source)
            .context("Failed to get relative path")?;

        // Skip if ignored
        if let Some(gi) = gitignore {
            let matched = gi.matched(relative_path, path.is_dir());
            if matched.is_ignore() {
                continue;
            }
        }

        // Skip the root directory itself
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        // Skip .stowignore file
        if relative_path.file_name() == Some(std::ffi::OsStr::new(".stowignore")) {
            continue;
        }

        let target_path = target.join(relative_path);

        if target_path.is_symlink() {
            // Verify it points to our source before removing
            let link_target = fs::read_link(&target_path)?;
            if link_target == path {
                fs::remove_file(&target_path).context(format!(
                    "Failed to remove symlink: {}",
                    target_path.display()
                ))?;
                println!("  Removed: {}", relative_path.display());
            }
        }
    }

    // Try to remove empty directories
    remove_empty_dirs(target)?;

    Ok(())
}

/// Recursively remove empty directories
pub fn remove_empty_dirs(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    // First, recursively process subdirectories
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Recursively try to remove subdirectories
                    let _ = remove_empty_dirs(&path);
                }
            }
        }
    }

    // Now try to remove this directory if it's empty
    match fs::remove_dir(dir) {
        Ok(_) => {
            println!("  Removed empty directory: {}", dir.display());
        }
        Err(_) => {
            // Directory not empty or other error, that's fine
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_stow_single_file() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("test.txt"), "hello").unwrap();

        stow_package(&source, &target, None).unwrap();

        assert!(target.join("test.txt").is_symlink());
        let link = fs::read_link(target.join("test.txt")).unwrap();
        assert_eq!(link, source.join("test.txt"));
    }

    #[test]
    fn test_stow_directory_structure() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(source.join("subdir")).unwrap();
        fs::write(source.join("file1.txt"), "content1").unwrap();
        fs::write(source.join("subdir/file2.txt"), "content2").unwrap();

        stow_package(&source, &target, None).unwrap();

        assert!(target.join("file1.txt").is_symlink());
        assert!(target.join("subdir").exists());
        assert!(target.join("subdir/file2.txt").is_symlink());
    }

    #[test]
    fn test_remove_package() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("test.txt"), "hello").unwrap();

        stow_package(&source, &target, None).unwrap();
        assert!(target.join("test.txt").exists());

        remove_package(&source, &target, None).unwrap();
        assert!(!target.join("test.txt").exists());
    }

    #[test]
    fn test_stowignore() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("include.txt"), "include").unwrap();
        fs::write(source.join("ignore.txt"), "ignore").unwrap();
        fs::write(source.join(".stowignore"), "ignore.txt").unwrap();

        let gitignore = load_ignore_rules(&source).unwrap();
        stow_package(&source, &target, gitignore.as_ref()).unwrap();

        assert!(target.join("include.txt").is_symlink());
        assert!(!target.join("ignore.txt").exists());
    }

    #[test]
    fn test_already_linked() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("test.txt"), "hello").unwrap();

        // Stow twice - should not error
        stow_package(&source, &target, None).unwrap();
        let result = stow_package(&source, &target, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_target_exists_error() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("test.txt"), "hello").unwrap();
        fs::write(target.join("test.txt"), "existing").unwrap();

        let result = stow_package(&source, &target, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Target already exists")
        );
    }

    #[test]
    fn test_remove_empty_directories() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(source.join("subdir")).unwrap();
        fs::write(source.join("subdir/file.txt"), "content").unwrap();

        stow_package(&source, &target, None).unwrap();
        assert!(target.join("subdir").exists());

        remove_package(&source, &target, None).unwrap();
        assert!(!target.join("subdir").exists());
    }

    #[test]
    fn test_ignore_directory() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(source.join("keep")).unwrap();
        fs::create_dir_all(source.join("ignore")).unwrap();
        fs::write(source.join("keep/file.txt"), "keep").unwrap();
        fs::write(source.join("ignore/file.txt"), "ignore").unwrap();
        fs::write(source.join(".stowignore"), "ignore/").unwrap();

        let gitignore = load_ignore_rules(&source).unwrap();
        stow_package(&source, &target, gitignore.as_ref()).unwrap();

        assert!(target.join("keep/file.txt").is_symlink());
        assert!(!target.join("ignore").exists());
    }
}
