#!/bin/bash
# Setup git hooks for xdg-config-stow

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_HOOKS_DIR="$(git rev-parse --git-dir)/hooks"

echo "Setting up git hooks..."

# Create symlink for pre-commit hook
if [ -L "$GIT_HOOKS_DIR/pre-commit" ]; then
    echo "✓ pre-commit hook already installed"
else
    ln -sf ../../scripts/pre-commit "$GIT_HOOKS_DIR/pre-commit"
    echo "✓ pre-commit hook installed"
fi

# Make sure the hook script is executable
chmod +x "$SCRIPT_DIR/pre-commit"

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will run:"
echo "  • cargo fmt --check"
echo "  • cargo clippy -- -D warnings"
echo "  • cargo test"
echo ""
echo "To bypass the hook (not recommended), use: git commit --no-verify"
