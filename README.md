# xdg-config-stow

An [XDG](https://specifications.freedesktop.org/basedir/latest/)-centric GNU stow replacement for managing dotfiles, written in Rust.

## Features

- Smart symlinking of dotfiles from a `.config/` directory to `XDG_CONFIG_HOME` (or `$HOME/.config`)
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

Run `xdg-config-stow` from your dotfiles repository root (the directory containing `.config/`).

### Stow a package

Link all files from `.config/fish` to `$HOME/.config/fish`:

```bash
xdg-config-stow fish
```

### Remove a stowed package

Remove symlinks for a previously stowed package:

```bash
xdg-config-stow --rm fish
```

### Dry run mode

Preview what changes would be made without actually making them:

```bash
# See what would be stowed
xdg-config-stow --dry-run fish

# See what would be removed
xdg-config-stow --rm --dry-run fish
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
  README.md
  setup.sh
```

## How it works

1. Detects the `.config/` directory in your current working directory
2. Resolves the target directory using `XDG_CONFIG_HOME` or falls back to `$HOME/.config`
3. Creates symlinks for all files in the specified package
4. Respects `.stowignore` files for excluding specific paths
5. Safely verifies symlink targets when removing packages

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

- **23 total tests** covering:
  - Core stowing/unstowing functionality
  - .stowignore pattern matching
  - Error handling and edge cases
  - XDG_CONFIG_HOME resolution
  - Complex directory structures
  - **Automatic migration safety** (6 dedicated safety tests)

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
