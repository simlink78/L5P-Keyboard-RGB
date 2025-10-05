# CLI-Only Migration Guide

This document describes the changes made to convert this application from a GUI-based tool to a pure command-line interface (CLI) application.

## Summary of Changes

The application has been significantly simplified by removing all GUI-related code and dependencies, reducing the codebase by approximately 1,300+ lines of code.

### What Was Removed

#### Dependencies (from Cargo.toml)
- `eframe` - GUI framework
- `egui-modal`, `egui_file`, `egui-notify` - GUI widgets and notifications
- `image` - Image processing for GUI icons
- `tray-icon` - System tray functionality
- `gtk` - Linux GUI toolkit
- `device_query` - Keyboard hotkey detection for GUI
- `winres` - Windows resource compiler for GUI icon
- `open` - Opening URLs/files (GUI feature)
- `winapi` console features - Windows console management for GUI

#### Source Files Removed
- `app/build.rs` - Windows resource build script
- `app/src/gui/` - Entire GUI module (~800 lines)
  - `mod.rs` - Main GUI application logic
  - `menu_bar.rs` - Menu bar implementation
  - `modals.rs` - Modal dialogs
  - `saved_items.rs` - Profile/effect management UI
  - `style.rs` - GUI styling
- `app/src/tray.rs` - System tray icon implementation
- `app/src/console.rs` - Windows console management
- `app/src/manager/effects/default_ui.rs` - Effect UI widgets

### What Changed

#### app/src/main.rs
**Before**: ~155 lines with GUI initialization, window management, tray icons
**After**: 17 lines - pure CLI entry point

Key changes:
- Removed `#![cfg_attr(not(test), windows_subsystem = "windows")]` (GUI-specific)
- Removed `start_ui()` function and all GUI initialization
- Removed `load_icon_data()` function
- Simplified to just call `cli::try_cli()` and handle errors

#### app/src/cli.rs
**Before**: ~265 lines with GUI/CLI mode switching
**After**: ~170 lines - pure CLI mode only

Key changes:
- Removed `--gui` and `--hide-window` flags
- Changed `arg_required_else_help` from commented out to `true` (CLI now requires a subcommand)
- Removed `GuiCommand::Start` variant, kept only `Exit`
- Removed `OutputType::NoArgs` (no longer needed without GUI fallback)
- Simplified `try_cli()` to only handle CLI mode
- Removed "No subcommands found, starting in GUI mode" message
- All commands now execute and exit (no persistent GUI session)

#### app/src/manager/mod.rs
- Removed `pub use effects::show_effect_ui;` export (GUI-only function)

#### app/src/manager/effects/mod.rs
- Removed entire `show_effect_ui()` function and related GUI code
- Removed `use` statements for `eframe`, `egui`, and `default_ui`
- Now only declares effect modules

### What Stayed the Same

The following functionality remains fully intact:

1. **All Effects** - Static, Breath, Smooth, Wave, Swipe, Lightning, AmbientLight, Disco, Christmas, Fade, Temperature, etc.
2. **Profile Management** - Save and load profiles to/from JSON files
3. **Custom Effects** - Create and load custom effects from JSON files
4. **Settings Persistence** - `persist.rs` module for saving/loading settings
5. **Driver Integration** - All keyboard communication via `legion-rgb-driver`
6. **CLI Commands** - All original CLI commands work identically:
   - `set` - Apply effects with various options
   - `list` - List available effects
   - `load-profile` - Load a profile from file
   - `custom-effect` - Load a custom effect from file

## Migration for Users

### Before (v0.20.7)
```bash
# Could run without arguments to start GUI
./legion-kb-rgb

# Or use CLI mode explicitly
./legion-kb-rgb set -e Static -c 255,0,0,255,0,0,255,0,0,255,0,0
```

### After (CLI-only)
```bash
# Must provide a command (shows help otherwise)
./legion-kb-rgb --help

# Use CLI commands (same as before)
./legion-kb-rgb set -e Static -c 255,0,0,255,0,0,255,0,0,255,0,0
```

### Key Differences

1. **No GUI** - The application will not open a window when double-clicked
2. **Command Required** - Running without arguments shows help instead of starting GUI
3. **No System Tray** - No system tray icon (the app exits after executing commands)
4. **No Hotkeys** - No Windows/Meta+RAlt profile cycling (was a GUI-only feature)

## Benefits of CLI-Only Approach

1. **Smaller Binary** - Significantly reduced executable size without GUI dependencies
2. **Fewer Dependencies** - Easier to build and maintain
3. **Better for Automation** - Perfect for scripts, systemd services, startup scripts
4. **Lower Resource Usage** - No GUI framework overhead
5. **Simpler Codebase** - ~1,300 fewer lines of code to maintain
6. **Cross-Platform Simplicity** - No platform-specific GUI code

## Example Usage

See the main README.md for detailed examples. Quick reference:

```bash
# List effects
legion-kb-rgb list

# Set static red
legion-kb-rgb set -e Static -c 255,0,0,255,0,0,255,0,0,255,0,0

# Use smooth wave effect
legion-kb-rgb set -e SmoothWave -s 4 -b 2 -d Left

# Save a profile
legion-kb-rgb set -e Static -c 255,0,0,255,0,0,255,0,0,255,0,0 --save red.json

# Load a profile
legion-kb-rgb load-profile -p red.json

# Load a custom effect
legion-kb-rgb custom-effect -p my-effect.json
```

## Configuration Files

Example configuration files are provided:
- `example-profile.json` - Example profile configuration
- `example-custom-effect.json` - Example custom effect configuration

The `LEGION_KEYBOARD_CONFIG` environment variable can still be used to specify a custom settings file location.

## For Developers

If you want to add the GUI back or create a separate GUI application:

1. The GUI code is still in git history (before this PR)
2. You could create a separate `legion-kb-rgb-gui` package
3. Both could share the `manager` and `driver` crates
4. The CLI is now a clean, simple interface that's easy to wrap with any GUI toolkit
