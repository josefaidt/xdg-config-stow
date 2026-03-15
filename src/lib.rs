use anyhow::{Context, Result, anyhow};
use colored::Colorize;
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

/// Check for conflicts before stowing
fn check_conflicts(
    source: &Path,
    target: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
) -> Result<Vec<(PathBuf, String)>> {
    let mut conflicts = Vec::new();

    // Only check top-level entries to avoid false positives with existing directory symlinks
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        // Skip .stowignore file
        if file_name == ".stowignore" {
            continue;
        }

        let relative_path = path
            .strip_prefix(source)
            .context("Failed to get relative path")?;

        // Check if ignored
        if let Some(gi) = gitignore {
            let matched = gi.matched(relative_path, path.is_dir());
            if matched.is_ignore() {
                continue;
            }
        }

        let target_path = target.join(relative_path);

        if path.is_dir() {
            // For directories, check if they can be symlinked as a whole
            if can_symlink_directory(&path, source, gitignore) {
                // Check if target is already correctly symlinked
                if target_path.is_symlink()
                    && let Ok(existing_link) = fs::read_link(&target_path)
                    && existing_link == path
                {
                    continue; // Already correct
                }
                // Check if target exists as something else
                if target_path.exists() && !target_path.is_symlink() {
                    conflicts.push((
                        target_path.clone(),
                        "directory exists (not a symlink)".to_string(),
                    ));
                } else if target_path.is_symlink()
                    && let Ok(existing_link) = fs::read_link(&target_path)
                {
                    conflicts.push((
                        target_path.clone(),
                        format!("symlink exists pointing to: {}", existing_link.display()),
                    ));
                }
            } else {
                // Need to check files inside recursively
                check_conflicts_recursive(&path, &target_path, source, gitignore, &mut conflicts)?;
            }
        } else {
            // For files, check if already correctly symlinked
            if target_path.is_symlink()
                && let Ok(existing_link) = fs::read_link(&target_path)
                && existing_link == path
            {
                continue; // Already correct
            }

            // Check if target exists as something else
            if target_path.exists() || target_path.is_symlink() {
                if target_path.is_symlink() {
                    if let Ok(existing_link) = fs::read_link(&target_path) {
                        conflicts.push((
                            target_path.clone(),
                            format!("symlink exists pointing to: {}", existing_link.display()),
                        ));
                    }
                } else {
                    conflicts.push((target_path.clone(), "file exists".to_string()));
                }
            }
        }
    }

    Ok(conflicts)
}

/// Recursively check for conflicts in directories that can't be symlinked as a whole
fn check_conflicts_recursive(
    source_dir: &Path,
    target_dir: &Path,
    source_root: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
    conflicts: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        let relative_path = path
            .strip_prefix(source_root)
            .context("Failed to get relative path")?;

        // Check if ignored
        if let Some(gi) = gitignore {
            let matched = gi.matched(relative_path, path.is_dir());
            if matched.is_ignore() {
                continue;
            }
        }

        let target_path = target_dir.join(path.file_name().unwrap());

        if path.is_dir() {
            if can_symlink_directory(&path, source_root, gitignore) {
                // Check directory symlink
                if target_path.is_symlink()
                    && let Ok(existing_link) = fs::read_link(&target_path)
                    && existing_link == path
                {
                    continue;
                }
                if target_path.exists() && !target_path.is_symlink() {
                    conflicts.push((
                        target_path.clone(),
                        "directory exists (not a symlink)".to_string(),
                    ));
                } else if target_path.is_symlink()
                    && let Ok(existing_link) = fs::read_link(&target_path)
                {
                    conflicts.push((
                        target_path.clone(),
                        format!("symlink exists pointing to: {}", existing_link.display()),
                    ));
                }
            } else {
                check_conflicts_recursive(&path, &target_path, source_root, gitignore, conflicts)?;
            }
        } else {
            // Check file
            if target_path.is_symlink()
                && let Ok(existing_link) = fs::read_link(&target_path)
                && existing_link == path
            {
                continue;
            }

            if target_path.exists() || target_path.is_symlink() {
                if target_path.is_symlink() {
                    if let Ok(existing_link) = fs::read_link(&target_path) {
                        conflicts.push((
                            target_path.clone(),
                            format!("symlink exists pointing to: {}", existing_link.display()),
                        ));
                    }
                } else {
                    conflicts.push((target_path.clone(), "file exists".to_string()));
                }
            }
        }
    }

    Ok(())
}

/// Check if a directory can be symlinked as a whole (no ignore rules apply to it)
fn can_symlink_directory(
    dir_path: &Path,
    source_root: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
) -> bool {
    if gitignore.is_none() {
        return true;
    }

    let gi = gitignore.unwrap();

    // Check if anything in this directory or its subdirectories is ignored
    for entry in WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if let Ok(relative_path) = path.strip_prefix(source_root) {
            let matched = gi.matched(relative_path, entry.file_type().is_dir());
            if matched.is_ignore() {
                return false; // Cannot symlink whole directory
            }
        }
    }

    true
}

/// Migrate from directory symlink to individual file symlinks if needed
fn migrate_directory_symlink(
    source_dir: &Path,
    target_dir: &Path,
    _source_root: &Path,
    _gitignore: Option<&ignore::gitignore::Gitignore>,
    dry_run: bool,
) -> Result<()> {
    // Check if target is a symlink pointing to our source
    if target_dir.is_symlink()
        && let Ok(link_target) = fs::read_link(target_dir)
        && link_target == source_dir
    {
        println!(
            "{}",
            format!(
                "Migrating directory symlink to file symlinks: {}",
                target_dir.display()
            )
            .yellow()
        );
        // Remove the directory symlink
        if dry_run {
            println!(
                "{} {}",
                "-".red(),
                format!("{}/", target_dir.display()).bright_red()
            );
            println!(
                "{} {}",
                "+".green(),
                format!("mkdir {}", target_dir.display()).dimmed()
            );
        } else {
            fs::remove_file(target_dir)?;
            // Create as real directory and stow contents
            fs::create_dir_all(target_dir)?;
        }
    }

    Ok(())
}

/// Stow a package by creating symlinks from source to target
pub fn stow_package(
    source: &Path,
    target: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
    dry_run: bool,
) -> Result<()> {
    let package_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("package");

    // Check if we can symlink the entire package directory
    if can_symlink_directory(source, source, gitignore) {
        // Check if target already exists and is correctly symlinked
        if target.is_symlink()
            && let Ok(existing_link) = fs::read_link(target)
            && existing_link == source
        {
            println!("Already linked: {}/", package_name);
            return Ok(());
        }

        // Check for conflicts
        if target.exists() && !target.is_symlink() {
            eprintln!(
                "\n❌ Cannot stow '{}' - target directory exists (not a symlink)\n",
                package_name
            );
            eprintln!("To resolve this issue, you can:\n");
            eprintln!("  1. Back up and remove the existing directory:");
            eprintln!(
                "     mv ~/.config/{} ~/.config/{}.backup",
                package_name, package_name
            );
            eprintln!("     xdg-config-stow {}\n", package_name);
            return Err(anyhow!("Target directory exists"));
        } else if target.is_symlink()
            && let Ok(existing_link) = fs::read_link(target)
        {
            eprintln!(
                "\n❌ Cannot stow '{}' - symlink exists pointing to: {}\n",
                package_name,
                existing_link.display()
            );
            eprintln!("To resolve this issue, you can:\n");
            eprintln!("  1. Remove the existing symlink:");
            eprintln!("     rm ~/.config/{}", package_name);
            eprintln!("     xdg-config-stow {}\n", package_name);
            return Err(anyhow!("Conflicting symlink exists"));
        }

        // Create parent directory if needed
        if let Some(parent) = target.parent()
            && !parent.exists()
        {
            if dry_run {
                println!(
                    "{} {}",
                    "+".green(),
                    format!("mkdir {}", parent.display()).dimmed()
                );
            } else {
                fs::create_dir_all(parent).context("Failed to create parent directory")?;
            }
        }

        // Create symlink to entire package directory
        if dry_run {
            println!(
                "{} {} -> {}",
                "+".green(),
                format!("{}/", package_name).bright_green(),
                source.display().to_string().dimmed()
            );
        } else {
            #[cfg(unix)]
            std::os::unix::fs::symlink(source, target).context(format!(
                "Failed to create package symlink: {}",
                target.display()
            ))?;

            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(source, target)?;

            println!("Linked package directory: {}/", package_name);
        }
        return Ok(());
    }

    // Can't symlink entire package, need to create directory and stow contents

    // Check if target is currently a package-level symlink to our source
    // If so, migrate it to individual file symlinks
    if target.is_symlink()
        && let Ok(existing_link) = fs::read_link(target)
        && existing_link == source
    {
        println!(
            "{}",
            format!(
                "Migrating package symlink to individual symlinks: {}/",
                package_name
            )
            .yellow()
        );
        // Remove the package-level symlink
        if dry_run {
            println!(
                "{} {}",
                "-".red(),
                format!("{}/", package_name).bright_red()
            );
            println!(
                "{} {}",
                "+".green(),
                format!("mkdir {}", target.display()).dimmed()
            );
        } else {
            fs::remove_file(target)?;
            // Create as real directory
            fs::create_dir_all(target)?;
        }
    } else if !target.exists() {
        if dry_run {
            println!(
                "{} {}",
                "+".green(),
                format!("mkdir {}", target.display()).dimmed()
            );
        } else {
            fs::create_dir_all(target).context("Failed to create target directory")?;
        }
    }

    // Check for conflicts first
    let conflicts = check_conflicts(source, target, gitignore)?;
    if !conflicts.is_empty() {
        eprintln!(
            "\n❌ Cannot stow '{}' - conflicts detected:\n",
            package_name
        );
        for (path, reason) in &conflicts {
            eprintln!("  • {} ({})", path.display(), reason);
        }
        eprintln!("\nTo resolve this issue, you can:\n");
        eprintln!("  1. Remove conflicting files manually:");
        eprintln!("     rm <file>");
        eprintln!("\n  2. Unstow first if previously stowed:");
        eprintln!("     xdg-config-stow --rm {}", package_name);
        eprintln!("\n  3. Back up and remove conflicts:");
        eprintln!(
            "     mv ~/.config/{} ~/.config/{}.backup",
            package_name, package_name
        );
        eprintln!("     xdg-config-stow {}\n", package_name);

        return Err(anyhow!(
            "{} conflict{} found",
            conflicts.len(),
            if conflicts.len() == 1 { "" } else { "s" }
        ));
    }

    // Migrate from directory symlinks if needed (for subdirectories)
    migrate_directory_symlink(source, target, source, gitignore, dry_run)?;

    // Stow contents
    stow_directory_contents(source, target, source, gitignore, dry_run)?;

    Ok(())
}

/// Recursively stow directory contents (used when directory can't be symlinked as a whole)
fn stow_directory_contents(
    source_dir: &Path,
    target_dir: &Path,
    source_root: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
    dry_run: bool,
) -> Result<()> {
    // Create target directory if it doesn't exist
    if !target_dir.exists() {
        if dry_run {
            println!(
                "{} {}",
                "+".green(),
                format!("mkdir {}", target_dir.display()).dimmed()
            );
        } else {
            fs::create_dir_all(target_dir)?;
            println!("Created directory: {}", target_dir.display());
        }
    }

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        let relative_path = path
            .strip_prefix(source_root)
            .context("Failed to get relative path")?;

        // Check if ignored
        if let Some(gi) = gitignore {
            let matched = gi.matched(relative_path, path.is_dir());
            if matched.is_ignore() {
                println!("Ignoring: {}", relative_path.display());
                continue;
            }
        }

        let target_path = target_dir.join(path.file_name().unwrap());

        if path.is_dir() {
            // Check if we can symlink this subdirectory
            if can_symlink_directory(&path, source_root, gitignore) {
                if target_path.is_symlink()
                    && let Ok(existing_link) = fs::read_link(&target_path)
                    && existing_link == path
                {
                    println!("Already linked: {}/", relative_path.display());
                    continue;
                }

                if !target_path.exists() {
                    if dry_run {
                        println!(
                            "{} {} -> {}",
                            "+".green(),
                            format!("{}/", relative_path.display()).bright_green(),
                            path.display().to_string().dimmed()
                        );
                    } else {
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(&path, &target_path)?;

                        #[cfg(windows)]
                        std::os::windows::fs::symlink_dir(&path, &target_path)?;

                        println!("Linked directory: {}/", relative_path.display());
                    }
                }
            } else {
                // Need to recurse
                migrate_directory_symlink(&path, &target_path, source_root, gitignore, dry_run)?;
                stow_directory_contents(&path, &target_path, source_root, gitignore, dry_run)?;
            }
        } else {
            if target_path.is_symlink()
                && let Ok(existing_link) = fs::read_link(&target_path)
                && existing_link == path
            {
                println!("Already linked: {}", relative_path.display());
                continue;
            }

            if !target_path.exists() {
                if dry_run {
                    println!(
                        "{} {} -> {}",
                        "+".green(),
                        relative_path.display().to_string().bright_green(),
                        path.display().to_string().dimmed()
                    );
                } else {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&path, &target_path)?;

                    #[cfg(windows)]
                    std::os::windows::fs::symlink_file(&path, &target_path)?;

                    println!("Linked: {}", relative_path.display());
                }
            }
        }
    }

    Ok(())
}

/// Stow a single file by creating a symlink from source to target
pub fn stow_single_file(source: &Path, target: &Path, dry_run: bool) -> Result<()> {
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    // Check if already correctly symlinked
    if target.is_symlink() {
        if let Ok(existing_link) = fs::read_link(target) {
            if existing_link == source {
                println!("Already linked: {}", file_name);
                return Ok(());
            }
            eprintln!(
                "\n❌ Cannot stow '{}' - symlink exists pointing to: {}\n",
                file_name,
                existing_link.display()
            );
            eprintln!("To resolve this issue, you can:\n");
            eprintln!("  1. Remove the existing symlink:");
            eprintln!("     rm ~/.config/{}", file_name);
            eprintln!("     xdg-config-stow {}\n", file_name);
            return Err(anyhow!("Conflicting symlink exists"));
        }
    }

    // Check if target exists as a real file
    if target.exists() {
        eprintln!(
            "\n❌ Cannot stow '{}' - file exists (not a symlink)\n",
            file_name
        );
        eprintln!("To resolve this issue, you can:\n");
        eprintln!("  1. Back up and remove the existing file:");
        eprintln!(
            "     mv ~/.config/{} ~/.config/{}.backup",
            file_name, file_name
        );
        eprintln!("     xdg-config-stow {}\n", file_name);
        return Err(anyhow!("Target file exists"));
    }

    // Create parent directory if needed
    if let Some(parent) = target.parent()
        && !parent.exists()
    {
        if dry_run {
            println!(
                "{} {}",
                "+".green(),
                format!("mkdir {}", parent.display()).dimmed()
            );
        } else {
            fs::create_dir_all(parent).context("Failed to create parent directory")?;
        }
    }

    if dry_run {
        println!(
            "{} {} -> {}",
            "+".green(),
            file_name.bright_green(),
            source.display().to_string().dimmed()
        );
    } else {
        #[cfg(unix)]
        std::os::unix::fs::symlink(source, target).context(format!(
            "Failed to create symlink: {}",
            target.display()
        ))?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_file(source, target)?;

        println!("Linked: {}", file_name);
    }

    Ok(())
}

/// Remove a stowed single file by deleting the symlink if it points to source
pub fn remove_single_file(source: &Path, target: &Path, dry_run: bool) -> Result<()> {
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    if !target.exists() && !target.is_symlink() {
        return Err(anyhow!(
            "Target file does not exist: {}",
            target.display()
        ));
    }

    if target.is_symlink() {
        let link_target = fs::read_link(target)?;
        if link_target == source {
            if dry_run {
                println!("{} {}", "-".red(), file_name.bright_red());
            } else {
                fs::remove_file(target).context(format!(
                    "Failed to remove symlink: {}",
                    target.display()
                ))?;
                println!("Removed: {}", file_name);
            }
            return Ok(());
        }
        return Err(anyhow!(
            "Target symlink does not point to source: {}",
            target.display()
        ));
    }

    Err(anyhow!("Target is not a symlink: {}", target.display()))
}

/// Remove a stowed package by deleting symlinks that point to source
pub fn remove_package(
    source: &Path,
    target: &Path,
    gitignore: Option<&ignore::gitignore::Gitignore>,
    dry_run: bool,
) -> Result<()> {
    if !target.exists() {
        return Err(anyhow!(
            "Target package does not exist: {}",
            target.display()
        ));
    }

    let package_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("package");

    // Check if the entire target is a symlink to our source (package-level symlink)
    if target.is_symlink()
        && let Ok(link_target) = fs::read_link(target)
        && link_target == source
    {
        if dry_run {
            println!(
                "{} {}",
                "-".red(),
                format!("{}/", package_name).bright_red()
            );
        } else {
            fs::remove_file(target).context(format!(
                "Failed to remove package symlink: {}",
                target.display()
            ))?;
            println!("Removed package symlink: {}/", package_name);
        }
        return Ok(());
    }

    // Otherwise, walk through source directory to find what should be removed
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
                if dry_run {
                    let display_path = if path.is_dir() {
                        format!("{}/", relative_path.display())
                    } else {
                        relative_path.display().to_string()
                    };
                    println!("{} {}", "-".red(), display_path.bright_red());
                } else {
                    fs::remove_file(&target_path).context(format!(
                        "Failed to remove symlink: {}",
                        target_path.display()
                    ))?;
                    println!("Removed: {}", relative_path.display());
                }
            }
        }
    }

    // Try to remove empty directories
    if !dry_run {
        remove_empty_dirs(target)?;
    } else {
        println!("{} {}", "-".red(), "empty directories".dimmed());
    }

    Ok(())
}

/// Recursively remove empty directories
pub fn remove_empty_dirs(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    // First, recursively process subdirectories
    if dir.is_dir()
        && let Ok(entries) = fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively try to remove subdirectories
                let _ = remove_empty_dirs(&path);
            }
        }
    }

    // Now try to remove this directory if it's empty
    match fs::remove_dir(dir) {
        Ok(_) => {
            println!("Removed empty directory: {}", dir.display());
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

        stow_package(&source, &target, None, false).unwrap();

        // With no ignore rules, entire package directory should be symlinked
        assert!(target.is_symlink());
        let link = fs::read_link(&target).unwrap();
        assert_eq!(link, source);
    }

    #[test]
    fn test_stow_directory_structure() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(source.join("subdir")).unwrap();
        fs::write(source.join("file1.txt"), "content1").unwrap();
        fs::write(source.join("subdir/file2.txt"), "content2").unwrap();

        stow_package(&source, &target, None, false).unwrap();

        // With no ignore rules, the entire package directory should be symlinked
        assert!(target.is_symlink());
        let link_target = fs::read_link(&target).unwrap();
        assert_eq!(link_target, source);
    }

    #[test]
    fn test_remove_package() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("test.txt"), "hello").unwrap();

        stow_package(&source, &target, None, false).unwrap();
        assert!(target.is_symlink());

        remove_package(&source, &target, None, false).unwrap();
        assert!(!target.exists());
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
        stow_package(&source, &target, gitignore.as_ref(), false).unwrap();

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
        stow_package(&source, &target, None, false).unwrap();
        assert!(target.is_symlink());
        let result = stow_package(&source, &target, None, false);
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

        let result = stow_package(&source, &target, None, false);
        assert!(result.is_err());
        // With package-level symlinking, when target exists as a directory, we get "Target directory exists"
        assert!(result.unwrap_err().to_string().contains("directory exists"));
    }

    #[test]
    fn test_remove_empty_directories() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(source.join("subdir")).unwrap();
        fs::write(source.join("subdir/file.txt"), "content").unwrap();

        stow_package(&source, &target, None, false).unwrap();
        assert!(target.is_symlink());

        remove_package(&source, &target, None, false).unwrap();
        assert!(!target.exists());
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
        stow_package(&source, &target, gitignore.as_ref(), false).unwrap();

        // With ignore rules, we can't symlink the whole package, so stow contents
        assert!(target.is_dir());
        assert!(!target.is_symlink());
        assert!(target.join("keep").is_symlink());
        assert!(!target.join("ignore").exists());
    }
}
