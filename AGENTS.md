# AGENTS.md

xdg-config-stow is a xdg-centric GNU stow replacement written in Rust. Its primary function is to be executed within a dotfiles repository containing a `.config/` directory, symlinking packages to `XDG_CONFIG_HOME` or `$HOME/.config`.

## Documentation

- **README.md** — user-facing docs, usage, installation
- **TESTS.md** — test documentation
- **CLAUDE.md** — full project context and architecture reference

## Working on GitHub Issues

When addressing a task via a GitHub issue, always update all relevant documentation before closing the task:

- **AGENTS.md** — update if architecture, workflows, or agent guidance changes
- **README.md** — update if user-facing behavior, flags, or usage changes
- **TESTS.md** — update if tests are added, removed, or their coverage changes

Keep these files in sync with the implementation so the project context stays accurate.

## Development

```bash
cargo build
cargo test
cargo test -- --nocapture  # with output
```

## Architecture

- **src/lib.rs** — core library (stow_package, remove_package, ignore handling)
- **src/main.rs** — CLI interface via clap
- **tests/integration_tests.rs** — integration tests

## Key Behaviors

- Symlinks files from `.config/<package>/` into `XDG_CONFIG_HOME` (or `$HOME/.config`)
- Supports single files: `.config/starship.toml` → `$HOME/.config/starship.toml` when argument is a file
- Respects `.stowignore` files (gitignore-style patterns) per package (directories only)
- `--rm` flag safely removes stowed symlinks and cleans up empty directories
- Idempotent: re-stowing an already-linked package or file is safe
- Will not overwrite existing non-symlink files
