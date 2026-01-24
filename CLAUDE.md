# CLAUDE.MD

xdg-config-stow is a xdg-centric GNU stow replacement. Its primary function is to be executed within a dotfiles repository cloned from git that contains a `.config/` directory.

## Product Vision

A simple, safe, and reliable tool for managing dotfiles using symlinks. Unlike GNU stow which is designed for general-purpose symlinking, xdg-config-stow is specifically built for XDG-compliant dotfile management.

## Example Directory Structure

```
my-dotfiles-repo/
  .config/
    fish/
      config.fish
      functions/
        fish_prompt.fish
      .stowignore
    nvim/
      init.lua
      lua/
        config/
  readme.md
  setup.sh
```

## Implementation

xdg-config-stow is written in Rust and smartly handles symlinks to XDG_CONFIG_HOME or $HOME/.config

### Core Features ✅

All original requirements have been implemented:

1. ✅ **Package Symlinking**: When executed within a directory containing `.config/`, symlinks individual "packages" to XDG_CONFIG_HOME or $HOME/.config
2. ✅ **Ignore File Support**: `.stowignore` files following gitignore spec for excluding files/directories
3. ✅ **Removal**: `--rm` flag for safely removing stowed packages
4. ✅ **XDG Compliance**: Respects XDG_CONFIG_HOME environment variable with fallback to $HOME/.config
5. ✅ **Safety**: Verifies symlinks before removal, prevents overwriting existing files
6. ✅ **Idempotent**: Re-stowing same package is safe and shows "Already linked" for existing symlinks

### Architecture

- **src/lib.rs**: Core library functions (stow_package, remove_package, ignore handling)
- **src/main.rs**: CLI interface using clap
- **tests/**: Comprehensive unit and integration tests

### Dependencies

- **clap**: CLI argument parsing with derive macros
- **ignore**: Gitignore-style pattern matching for .stowignore
- **anyhow**: Error handling and context
- **walkdir**: Directory traversal

## Usage

Run `xdg-config-stow` from your dotfiles repository root (the directory containing `.config/`).

### Basic Commands

```bash
# Stow fish config (creates symlinks)
xdg-config-stow fish

# Remove stowed fish config (removes symlinks)
xdg-config-stow --rm fish
# or
xdg-config-stow -r fish
```

### Using .stowignore

Create `.config/fish/.stowignore`:

```gitignore
# Ignore completions directory (may differ per machine)
completions/

# Ignore fish variables (machine-specific)
fish_variables

# Patterns work like gitignore
*.local
temp/
```

When you run `xdg-config-stow fish`, ignored files/directories will be skipped:
```
  Ignoring: completions
  Ignoring: fish_variables
  Linked: config.fish
  Linked: functions/fish_prompt.fish
Successfully stowed package 'fish'
```

### Environment Variables

```bash
# Use custom config directory
XDG_CONFIG_HOME=/path/to/config xdg-config-stow fish

# Falls back to $HOME/.config if not set
xdg-config-stow fish
```

## Testing

The project has comprehensive test coverage:

### Test Statistics
- **17 total tests** (all passing ✅)
- **8 unit tests** in src/lib.rs
- **9 integration tests** in tests/integration_tests.rs

### Test Coverage Includes
- Core stowing/unstowing functionality
- .stowignore pattern matching (files and directories)
- Error handling (missing dirs, conflicting files, etc.)
- XDG_CONFIG_HOME resolution
- Complex nested directory structures
- Platform-specific path handling (macOS /var symlinks)
- Idempotent operations
- Empty directory cleanup

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_stow_single_file
```

See TESTS.md for detailed test documentation.

## Installation

```bash
# Build from source
cargo build --release

# Binary will be in target/release/xdg-config-stow

# Install locally
cargo install --path .
```

## How It Works

1. **Validates Environment**: Checks for `.config/` directory and specified package
2. **Resolves Target**: Uses `XDG_CONFIG_HOME` or falls back to `$HOME/.config`
3. **Loads Ignore Rules**: Parses `.stowignore` if present
4. **Creates Symlinks**: Walks directory tree, creating symlinks for files and directories for structure
5. **Handles Errors**: Clear error messages for conflicts or missing files
6. **Cleanup on Remove**: Removes symlinks and empty directories when using `--rm`

## Safety Features

- Only removes symlinks that point to the source package
- Won't overwrite existing files (returns error instead)
- Creates parent directories as needed
- Verifies `.config/` directory exists before operating
- Respects ignore patterns to avoid stowing sensitive/machine-specific files

## Future Considerations

Potential enhancements (not yet implemented):

- **Conflict Resolution**: Interactive mode for handling existing files
- **Backup Creation**: Optional backup before stowing
- **Multiple Package Stowing**: `xdg-config-stow fish nvim tmux`
- **Global Ignore File**: Repository-level .stowignore in addition to package-level
- **Dry Run Mode**: Preview what would be stowed without making changes
- **Verification**: Check integrity of existing stowed packages
- **Logging Levels**: Quiet/verbose modes
- **Package Dependencies**: Stow package B when package A is stowed

## Development

See README.md and TESTS.md for development setup and test information.

### Quick Start

```bash
# Clone and build
git clone <repo-url>
cd xdg-stow
cargo build

# Run tests
cargo test

# Try it out (from your dotfiles repo)
cd ~/dotfiles
xdg-config-stow fish
```

## License

MIT
