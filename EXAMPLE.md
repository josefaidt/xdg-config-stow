# Example Usage

## Setup Example

Let's say you have a dotfiles repository with this structure:

```
dotfiles/
  .config/
    fish/
      config.fish
      functions/
        fish_prompt.fish
      completions/
        custom.fish
      .stowignore
    nvim/
      init.lua
      lua/
        config/
          options.lua
```

### Example 1: Stow fish config

```bash
cd dotfiles
xdg-config-stow fish
```

This will create symlinks from `dotfiles/.config/fish/*` to `$HOME/.config/fish/*`.

Output:
```
Created directory: functions
Linked: config.fish
Linked: functions/fish_prompt.fish
Created directory: completions
Linked: completions/custom.fish
Successfully stowed package 'fish'
```

### Example 2: Using .stowignore

Create `dotfiles/.config/fish/.stowignore`:

```gitignore
# Don't sync completions across machines
completions/

# Don't sync fish variables (machine-specific)
fish_variables
```

Now when you run:
```bash
xdg-config-stow fish
```

The completions directory and fish_variables file will be skipped:
```
Ignoring: completions
Ignoring: fish_variables
Created directory: functions
Linked: config.fish
Linked: functions/fish_prompt.fish
Successfully stowed package 'fish'
```

### Example 3: Remove stowed package

To remove the symlinks:

```bash
xdg-config-stow --rm fish
```

Output:
```
Removed: config.fish
Removed: functions/fish_prompt.fish
Removed empty directory: /Users/you/.config/fish/functions
Removed empty directory: /Users/you/.config/fish
Successfully removed package 'fish'
```

## Testing the Build

You can test the application locally by creating a test directory structure:

```bash
# Create test structure
mkdir -p test-dotfiles/.config/fish/functions
echo 'echo "Hello from fish!"' > test-dotfiles/.config/fish/config.fish
echo 'function fish_prompt; echo "> "; end' > test-dotfiles/.config/fish/functions/fish_prompt.fish

# Test stowing (with a test XDG_CONFIG_HOME to avoid affecting your real config)
cd test-dotfiles
export XDG_CONFIG_HOME=/tmp/test-config
../target/debug/xdg-config-stow fish

# Check the symlinks
ls -la /tmp/test-config/fish/

# Clean up
../target/debug/xdg-config-stow --rm fish
```
