# xdg-config-stow

An [XDG](https://specifications.freedesktop.org/basedir/latest/)-centric GNU stow replacement for managing dotfiles, written in Rust.

## Features

- Smart symlinking of dotfiles from a `.config/` directory to `XDG_CONFIG_HOME` (or `$HOME/.config`)
- Support for user-generated scripts from `.local/bin/` to `$HOME/.local/bin` (derived from `XDG_DATA_HOME`)
- Support for `.stowignore` files using gitignore-style patterns
- Easy removal of stowed packages with the `--rm` flag
- Safe operation - verifies symlinks before removal

## Installation

Install from crates.io:

```bash
cargo install xdg-config-stow
```

Or build from source:

```bash
cargo build --release
# Binary will be in target/release/xdg-config-stow
```

## Usage

Run `xdg-config-stow` from your dotfiles repository root (the directory containing `.config/` and/or `.local/bin/`).

### Stow a config package

Link all files from `.config/fish` to `$HOME/.config/fish`:

```bash
xdg-config-stow fish
```

### Stow a user script

Link a script from `.local/bin/my-script` to `$HOME/.local/bin/my-script`:

```bash
xdg-config-stow my-script
```

The tool first looks for the package in `.config/`, then falls back to `.local/bin/`. The bin target directory is derived from `XDG_DATA_HOME` (defaulting to `$HOME/.local/bin`).

### Remove a stowed package

Remove symlinks for a previously stowed package:

```bash
xdg-config-stow --rm fish
xdg-config-stow --rm my-script
```

### Dry run mode

Preview what changes would be made without actually making them. Output uses colored diff-like syntax with `+` for additions and `-` for removals:

```bash
# See what would be stowed
xdg-config-stow --dry-run fish
# Output:
# DRY RUN: No changes will be made
#
# + config.fish -> /path/to/dotfiles/.config/fish/config.fish
# + functions/ -> /path/to/dotfiles/.config/fish/functions

# See what would be removed
xdg-config-stow --rm --dry-run fish
# Output:
# DRY RUN: No changes will be made
#
# - config.fish
# - functions/
```

### Ignoring files

Create a `.stowignore` file inside your package directory (e.g., `.config/fish/.stowignore`) using gitignore syntax:

```gitignore
# Ignore completions directory
completions/

# Ignore specific files
fish_variables
```

**Note**: If you add a `.stowignore` file after initially stowing a package, simply re-run the stow command to automatically update the symlinks:

```bash
xdg-config-stow fish  # Automatically migrates and respects new ignore rules
```

## Example Directory Structure

```
my-dotfiles-repo/
  .config/
    fish/
      config.fish
      functions/
      .stowignore
    nvim/
      init.lua
      lua/
  .local/
    bin/
      my-script
      update-dots
  README.md
  setup.sh
```

## How it works

1. Detects `.config/` and/or `.local/bin/` in your current working directory
2. For config packages: resolves target using `XDG_CONFIG_HOME` or falls back to `$HOME/.config`
3. For bin scripts: resolves target using the parent of `XDG_DATA_HOME` joined with `bin`, or falls back to `$HOME/.local/bin`
4. Creates symlinks for all files in the specified package
5. Respects `.stowignore` files for excluding specific paths
6. Safely verifies symlink targets when removing packages

When both `.config/<name>` and `.local/bin/<name>` exist, the config package takes priority.

## Requirements

- Rust 1.85 or later (for building)
- Unix-like operating system (Linux, macOS, BSD)
- Write access to `XDG_CONFIG_HOME` or `$HOME/.config`

## Development

### Running Tests

The project includes comprehensive unit and integration tests:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_stow_single_file
```

See [TESTS.md](TESTS.md) for detailed test coverage information.

### Test Coverage

- **29 total tests** covering:
  - Core stowing/unstowing functionality
  - .stowignore pattern matching
  - Error handling and edge cases
  - XDG_CONFIG_HOME resolution
  - Complex directory structures
  - **Automatic migration safety** (6 dedicated safety tests)
  - **Bin support** (6 dedicated tests)

## Contributing

Contributions are welcome! Before submitting a pull request:

1. Ensure your code is properly formatted: `cargo fmt`
2. Check for linting issues: `cargo clippy -- -D warnings`
3. Run all tests: `cargo test`

### Git Hooks (Optional)

To automatically run these checks before every commit, you can set up git hooks:

```bash
./scripts/setup-hooks.sh
```

This will install a pre-commit hook that runs formatting checks, clippy, and tests before allowing commits. You can bypass the hook when needed with `git commit --no-verify`.

## License

MIT
