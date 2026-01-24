# Test Suite

This project includes comprehensive unit and integration tests to ensure xdg-config-stow works correctly.

## Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_stow_single_file
```

## Test Coverage

### Unit Tests (src/lib.rs)

**8 tests** covering core library functions:

1. `test_stow_single_file` - Basic symlink creation for a single file
2. `test_stow_directory_structure` - Handling nested directories and multiple files
3. `test_remove_package` - Symlink removal functionality
4. `test_stowignore` - .stowignore file pattern matching
5. `test_already_linked` - Idempotent stowing (re-stowing same package)
6. `test_target_exists_error` - Error when target file exists but isn't our symlink
7. `test_remove_empty_directories` - Cleanup of empty directories after removal
8. `test_ignore_directory` - Ignoring entire directories via .stowignore

### Integration Tests (tests/integration_tests.rs)

**9 tests** testing the CLI end-to-end:

1. `test_missing_config_directory` - Error handling when .config doesn't exist
2. `test_missing_package` - Error handling for non-existent packages
3. `test_stow_and_remove_package` - Full workflow of stowing and removing
4. `test_stow_with_ignore_file` - .stowignore integration with CLI
5. `test_stow_already_linked` - CLI behavior when re-stowing
6. `test_stow_target_exists_error` - CLI error when target file exists
7. `test_remove_nonexistent_package` - Error when removing non-stowed package
8. `test_complex_directory_structure` - Deeply nested directory structures
9. `test_xdg_config_home_resolution` - Custom XDG_CONFIG_HOME handling

## Test Results

```
Unit tests:      8 passed
Integration:     9 passed
Total:          17 passed
```

## What's Tested

### Core Functionality
- ✅ Creating symlinks for files
- ✅ Creating directory structure
- ✅ Removing symlinks
- ✅ Cleaning up empty directories
- ✅ XDG_CONFIG_HOME environment variable resolution

### .stowignore Support
- ✅ Ignoring individual files
- ✅ Ignoring entire directories
- ✅ Gitignore-style pattern matching

### Error Handling
- ✅ Missing .config directory
- ✅ Missing package
- ✅ Existing files/symlinks
- ✅ Attempting to remove non-existent package

### Edge Cases
- ✅ Idempotent operations (re-stowing same package)
- ✅ Complex nested directory structures
- ✅ Platform-specific path handling (macOS /var symlink)

## CI/CD

The test suite is designed to be run in CI environments. All tests use temporary directories and don't require any system configuration.

## Future Test Ideas

- Performance tests with large directory structures
- Windows-specific symlink tests
- Concurrent stowing operations
- Symlink verification and repair
