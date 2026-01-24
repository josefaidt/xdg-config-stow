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

    // Verify symlinks were created
    assert!(target.join("fish/config.fish").is_symlink());
    assert!(target.join("fish/functions/prompt.fish").is_symlink());

    // Verify symlink targets are correct (canonicalize to handle macOS /var -> /private/var symlink)
    let link = fs::read_link(target.join("fish/config.fish")).unwrap();
    let link_canonical = link.canonicalize().unwrap();
    let expected_canonical = dotfiles
        .join(".config/fish/config.fish")
        .canonicalize()
        .unwrap();
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
    assert!(stderr.contains("Target already exists"));
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

    // Verify all symlinks were created
    assert!(target.join("nvim/init.lua").is_symlink());
    assert!(target.join("nvim/lua/config/options.lua").is_symlink());
    assert!(target.join("nvim/after/plugin/colors.lua").is_symlink());

    // Verify directories exist
    assert!(target.join("nvim/lua/config").is_dir());
    assert!(target.join("nvim/after/plugin").is_dir());
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
    assert!(custom_target.join("test/file.txt").is_symlink());
}
