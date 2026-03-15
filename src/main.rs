use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::Colorize;
use std::fs;
use xdg_config_stow::{get_xdg_bin_home, get_xdg_config_home, load_ignore_rules, remove_package, stow_package};

#[derive(Parser, Debug)]
#[command(name = "xdg-config-stow")]
#[command(about = "XDG-centric GNU stow replacement for dotfiles", long_about = None)]
struct Args {
    /// Package name to stow (e.g., fish, nvim)
    package: String,

    /// Remove stowed package instead of creating symlinks
    #[arg(long)]
    rm: bool,

    /// Show what would be done without making any changes
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Get current working directory
    let cwd = std::env::current_dir().context("Failed to get current working directory")?;

    // Check for .config or .local/bin directory in current directory
    let config_dir = cwd.join(".config");
    let bin_dir = cwd.join(".local").join("bin");

    let has_config_dir = config_dir.exists() && config_dir.is_dir();
    let has_bin_dir = bin_dir.exists() && bin_dir.is_dir();

    if !has_config_dir && !has_bin_dir {
        return Err(anyhow!(
            "No .config directory found in current directory. Please run this command from your dotfiles repository root."
        ));
    }

    // Resolve package source and target: prefer .config/<package>, fall back to .local/bin/<package>
    let config_source = config_dir.join(&args.package);
    let bin_source = bin_dir.join(&args.package);

    let (package_source, target_dir) = if has_config_dir && config_source.exists() {
        (config_source, get_xdg_config_home()?)
    } else if has_bin_dir && bin_source.exists() {
        (bin_source, get_xdg_bin_home()?)
    } else if has_config_dir {
        return Err(anyhow!(
            "Package '{}' not found in .config directory",
            args.package
        ));
    } else {
        return Err(anyhow!(
            "Package '{}' not found in .local/bin",
            args.package
        ));
    };

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).context("Failed to create target directory")?;
    }

    let target_package = target_dir.join(&args.package);

    // Load ignore rules if .stowignore exists
    let gitignore = load_ignore_rules(&package_source)?;

    if args.dry_run {
        println!("{}\n", "DRY RUN: No changes will be made".yellow().bold());
    }

    if args.rm {
        remove_package(
            &package_source,
            &target_package,
            gitignore.as_ref(),
            args.dry_run,
        )?;
        if !args.dry_run {
            println!("Successfully removed package '{}'", args.package);
        }
    } else {
        stow_package(
            &package_source,
            &target_package,
            gitignore.as_ref(),
            args.dry_run,
        )?;
        if !args.dry_run {
            println!("Successfully stowed package '{}'", args.package);
        }
    }

    Ok(())
}
