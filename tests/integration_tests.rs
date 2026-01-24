use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove "deps"
    path.push("xdg-config-stow");
    path
}

#[test]
fn test_missing_config_directory() {
    let temp = TempDir::new().unwrap();

    let output = Command::new(get_binary_path())
        .arg("fish")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No .config directory found"));
}

#[test]
fn test_missing_package() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".config")).unwrap();

    let output = Command::new(get_binary_path())
        .arg("nonexistent")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found in .config directory"));
}

#[test]
fn test_stow_and_remove_package() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source structure
    fs::create_dir_all(dotfiles.join(".config/fish/functions")).unwrap();
    fs::write(dotfiles.join(".config/fish/config.fish"), "# fish config").unwrap();
    fs::write(
        dotfiles.join(".config/fish/functions/prompt.fish"),
        "# prompt function",
    )
    .unwrap();

    // Stow the package
    let output = Command::new(get_binary_path())
        .arg("fish")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success(), "Stow command failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully stowed package 'fish'"));

    // Verify entire fish directory is symlinked (with no ignore rules, package-level symlink)
    assert!(target.join("fish").is_symlink());

    // Verify symlink target is correct (canonicalize to handle macOS /var -> /private/var symlink)
    let link = fs::read_link(target.join("fish")).unwrap();
    let link_canonical = link.canonicalize().unwrap();
    let expected_canonical = dotfiles.join(".config/fish").canonicalize().unwrap();
    assert_eq!(link_canonical, expected_canonical);

    // Remove the package
    let output = Command::new(get_binary_path())
        .arg("--rm")
        .arg("fish")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Remove command failed: {:?}",
        output
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully removed package 'fish'"));

    // Verify symlinks were removed
    assert!(!target.join("fish/config.fish").exists());
    assert!(!target.join("fish/functions/prompt.fish").exists());
}

#[test]
fn test_stow_with_ignore_file() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source structure
    fs::create_dir_all(dotfiles.join(".config/fish/completions")).unwrap();
    fs::write(dotfiles.join(".config/fish/config.fish"), "# fish config").unwrap();
    fs::write(
        dotfiles.join(".config/fish/completions/custom.fish"),
        "# custom completion",
    )
    .unwrap();
    fs::write(dotfiles.join(".config/fish/fish_variables"), "# variables").unwrap();

    // Create .stowignore file
    fs::write(
        dotfiles.join(".config/fish/.stowignore"),
        "completions/\nfish_variables\n",
    )
    .unwrap();

    // Stow the package
    let output = Command::new(get_binary_path())
        .arg("fish")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify only non-ignored files were stowed
    assert!(target.join("fish/config.fish").is_symlink());
    assert!(!target.join("fish/completions").exists());
    assert!(!target.join("fish/fish_variables").exists());
}

#[test]
fn test_stow_already_linked() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source structure
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/file.txt"), "content").unwrap();

    // Stow once
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Stow again - should succeed and show "Already linked"
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Already linked"));
}

#[test]
fn test_stow_target_exists_error() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source structure
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/file.txt"), "source content").unwrap();

    // Create existing file in target
    fs::create_dir_all(target.join("test")).unwrap();
    fs::write(target.join("test/file.txt"), "existing content").unwrap();

    // Try to stow - should fail
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("directory exists"));
}

#[test]
fn test_remove_nonexistent_package() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source structure
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/file.txt"), "content").unwrap();

    // Try to remove without stowing first
    let output = Command::new(get_binary_path())
        .arg("--rm")
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Target package does not exist"));
}

#[test]
fn test_complex_directory_structure() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup complex structure
    fs::create_dir_all(dotfiles.join(".config/nvim/lua/config")).unwrap();
    fs::create_dir_all(dotfiles.join(".config/nvim/after/plugin")).unwrap();
    fs::write(dotfiles.join(".config/nvim/init.lua"), "-- init").unwrap();
    fs::write(
        dotfiles.join(".config/nvim/lua/config/options.lua"),
        "-- options",
    )
    .unwrap();
    fs::write(
        dotfiles.join(".config/nvim/after/plugin/colors.lua"),
        "-- colors",
    )
    .unwrap();

    // Stow the package
    let output = Command::new(get_binary_path())
        .arg("nvim")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify entire nvim directory is symlinked (with no ignore rules, package-level symlink)
    assert!(target.join("nvim").is_symlink());

    // Verify symlink target is correct (canonicalize to handle macOS /var -> /private/var symlink)
    let link = fs::read_link(target.join("nvim")).unwrap();
    let link_canonical = link.canonicalize().unwrap();
    let expected_canonical = dotfiles.join(".config/nvim").canonicalize().unwrap();
    assert_eq!(link_canonical, expected_canonical);
}

#[test]
fn test_directory_symlink_migration() {
    // Test that when .stowignore is added after initial stow, re-stowing migrates from
    // directory symlink to individual file symlinks
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup structure
    fs::create_dir_all(dotfiles.join(".config/fish/functions")).unwrap();
    fs::write(dotfiles.join(".config/fish/config.fish"), "# config").unwrap();
    fs::write(
        dotfiles.join(".config/fish/functions/prompt.fish"),
        "# prompt",
    )
    .unwrap();
    fs::write(dotfiles.join(".config/fish/fish_variables"), "# variables").unwrap();

    // First stow - without .stowignore, entire functions directory should be symlinked
    let output = Command::new(get_binary_path())
        .arg("fish")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    // With no ignore rules, entire fish package should be symlinked
    assert!(target.join("fish").is_symlink());

    // Now add .stowignore to exclude fish_variables
    fs::write(
        dotfiles.join(".config/fish/.stowignore"),
        "fish_variables\n",
    )
    .unwrap();

    // Re-stow - should automatically migrate and respect new ignore rules
    let output = Command::new(get_binary_path())
        .arg("fish")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Migrating package symlink to individual symlinks"));

    // After re-stowing with .stowignore, fish directory should exist (not be a symlink)
    assert!(target.join("fish").is_dir());
    assert!(!target.join("fish").is_symlink());
    // files should be symlinked individually
    assert!(target.join("fish/config.fish").is_symlink());
    // functions directory should be symlinked (no ignore rules for it)
    assert!(target.join("fish/functions").is_symlink());
    // fish_variables should not exist because it's ignored
    assert!(!target.join("fish/fish_variables").exists());
}

#[test]
fn test_directory_to_file_symlinks_migration() {
    // Test migrating a directory symlink to file symlinks when ignore rules are added
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup structure with subdirectory
    fs::create_dir_all(dotfiles.join(".config/app/subdir")).unwrap();
    fs::write(dotfiles.join(".config/app/keep.txt"), "keep").unwrap();
    fs::write(dotfiles.join(".config/app/subdir/file1.txt"), "file1").unwrap();
    fs::write(dotfiles.join(".config/app/subdir/file2.txt"), "file2").unwrap();

    // First stow - subdir should be symlinked as a whole
    let output = Command::new(get_binary_path())
        .arg("app")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    // With no ignore rules, entire app package should be symlinked
    assert!(target.join("app").is_symlink());

    // Add .stowignore to exclude subdir/file2.txt
    fs::write(
        dotfiles.join(".config/app/.stowignore"),
        "subdir/file2.txt\n",
    )
    .unwrap();

    // Re-stow - should automatically migrate and create individual file symlinks
    let output = Command::new(get_binary_path())
        .arg("app")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Migrating package symlink to individual symlinks"));

    // After re-stowing with ignore rule: app should be a real directory, not a symlink
    assert!(target.join("app").is_dir());
    assert!(!target.join("app").is_symlink());

    // keep.txt should be symlinked
    assert!(target.join("app/keep.txt").is_symlink());
    // subdir should be a real directory (has ignore rules inside)
    assert!(target.join("app/subdir").is_dir());
    assert!(!target.join("app/subdir").is_symlink());
    // file1.txt should be symlinked, file2.txt should not exist
    assert!(target.join("app/subdir/file1.txt").is_symlink());
    assert!(!target.join("app/subdir/file2.txt").exists());
}

#[test]
fn test_xdg_config_home_resolution() {
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let custom_target = temp.path().join("custom_config");

    // Setup source structure
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/file.txt"), "content").unwrap();

    // Stow with custom XDG_CONFIG_HOME
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &custom_target)
        .output()
        .unwrap();

    assert!(output.status.success());
    // Entire test package should be symlinked
    assert!(custom_target.join("test").is_symlink());
}

#[test]
fn test_migration_safety_wrong_symlink() {
    // Test that migration only happens when symlink points to correct source
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");
    let other_source = temp.path().join("other");

    // Setup dotfiles structure
    fs::create_dir_all(dotfiles.join(".config/fish")).unwrap();
    fs::write(dotfiles.join(".config/fish/config.fish"), "# config").unwrap();

    // Setup other source that target is linked to
    fs::create_dir_all(&other_source).unwrap();
    fs::create_dir_all(&target).unwrap();

    // Create symlink pointing to wrong source
    #[cfg(unix)]
    std::os::unix::fs::symlink(&other_source, target.join("fish")).unwrap();

    // Add .stowignore to trigger migration attempt
    fs::write(dotfiles.join(".config/fish/.stowignore"), "completions/\n").unwrap();

    // Try to stow - should fail with conflict, not attempt migration
    let output = Command::new(get_binary_path())
        .arg("fish")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("symlink exists pointing to"));

    // Verify original symlink was NOT removed
    assert!(target.join("fish").is_symlink());
    let link = fs::read_link(target.join("fish")).unwrap();
    assert_eq!(link, other_source); // Still points to other source
}

#[test]
fn test_migration_with_conflicting_file() {
    // Test migration failure when a conflicting file exists
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/keep.txt"), "keep").unwrap();
    fs::write(dotfiles.join(".config/test/conflict.txt"), "source content").unwrap();

    // First stow - package-level symlink
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(target.join("test").is_symlink());

    // Manually break the symlink and create conflicting file
    fs::remove_file(target.join("test")).unwrap();
    fs::create_dir_all(target.join("test")).unwrap();
    fs::write(target.join("test/conflict.txt"), "existing content").unwrap();

    // Add .stowignore to trigger migration
    fs::write(dotfiles.join(".config/test/.stowignore"), "ignore.txt\n").unwrap();

    // Try to re-stow - should detect conflict and fail
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // When target is a directory (not a symlink), we get "Target directory exists"
    // This tests that we DON'T attempt migration when target isn't a symlink pointing to source
    assert!(stderr.contains("Target directory exists"));
}

#[test]
fn test_migration_preserves_correct_symlinks() {
    // Test that migration doesn't break if some files are already correctly symlinked
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/file1.txt"), "file1").unwrap();
    fs::write(dotfiles.join(".config/test/file2.txt"), "file2").unwrap();
    fs::write(dotfiles.join(".config/test/ignore.txt"), "ignore").unwrap();

    // First stow - package-level symlink
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(target.join("test").is_symlink());

    // Add .stowignore to trigger migration
    fs::write(dotfiles.join(".config/test/.stowignore"), "ignore.txt\n").unwrap();

    // Re-stow - should migrate successfully
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Migrating package symlink to individual symlinks"));

    // Verify all files are correctly symlinked
    assert!(target.join("test").is_dir());
    assert!(!target.join("test").is_symlink());
    assert!(target.join("test/file1.txt").is_symlink());
    assert!(target.join("test/file2.txt").is_symlink());
    assert!(!target.join("test/ignore.txt").exists());

    // Verify symlinks point to correct source (canonicalize for macOS /var vs /private/var)
    let link1 = fs::read_link(target.join("test/file1.txt"))
        .unwrap()
        .canonicalize()
        .unwrap();
    let expected1 = dotfiles
        .join(".config/test/file1.txt")
        .canonicalize()
        .unwrap();
    assert_eq!(link1, expected1);
}

#[test]
fn test_no_migration_when_not_needed() {
    // Test that we don't attempt migration when target is not a package symlink
    let temp = TempDir::new().unwrap();
    let dotfiles = temp.path().join("dotfiles");
    let target = temp.path().join("target");

    // Setup source
    fs::create_dir_all(dotfiles.join(".config/test")).unwrap();
    fs::write(dotfiles.join(".config/test/file.txt"), "content").unwrap();
    fs::write(dotfiles.join(".config/test/ignore.txt"), "ignore").unwrap();

    // Add .stowignore BEFORE first stow
    fs::write(dotfiles.join(".config/test/.stowignore"), "ignore.txt\n").unwrap();

    // First stow - should create individual symlinks, not package symlink
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should NOT contain migration message
    assert!(!stdout.contains("Migrating"));

    // Verify individual symlinks were created
    assert!(target.join("test").is_dir());
    assert!(!target.join("test").is_symlink());
    assert!(target.join("test/file.txt").is_symlink());
    assert!(!target.join("test/ignore.txt").exists());

    // Re-stow - should be idempotent with no migration
    let output = Command::new(get_binary_path())
        .arg("test")
        .current_dir(&dotfiles)
        .env("XDG_CONFIG_HOME", &target)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Migrating"));
    assert!(stdout.contains("Already linked"));
}
