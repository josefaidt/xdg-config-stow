use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::fs;
use xdg_config_stow::{get_xdg_config_home, load_ignore_rules, remove_package, stow_package};

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

    // Check for .config directory in current directory
    let config_dir = cwd.join(".config");
    if !config_dir.exists() || !config_dir.is_dir() {
        return Err(anyhow!(
            "No .config directory found in current directory. Please run this command from your dotfiles repository root."
        ));
    }

    // Check if the package exists in .config
    let package_source = config_dir.join(&args.package);
    if !package_source.exists() {
        return Err(anyhow!(
            "Package '{}' not found in .config directory",
            args.package
        ));
    }

    // Resolve target directory (XDG_CONFIG_HOME or $HOME/.config)
    let target_dir = get_xdg_config_home()?;
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).context("Failed to create target directory")?;
    }

    let target_package = target_dir.join(&args.package);

    // Load ignore rules if .stowignore exists
    let gitignore = load_ignore_rules(&package_source)?;

    if args.dry_run {
        println!("DRY RUN: No changes will be made\n");
    }

    if args.rm {
        remove_package(&package_source, &target_package, gitignore.as_ref(), args.dry_run)?;
        if !args.dry_run {
            println!("Successfully removed package '{}'", args.package);
        }
    } else {
        stow_package(&package_source, &target_package, gitignore.as_ref(), args.dry_run)?;
        if !args.dry_run {
            println!("Successfully stowed package '{}'", args.package);
        }
    }

    Ok(())
}
