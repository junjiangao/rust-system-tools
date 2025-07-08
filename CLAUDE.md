# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build
-   **Basic build**: `cargo build`
-   **Release build**: `cargo build --release`

### Run
-   **CLI mode (mount)**: `./target/release/rust-system-tools mount -i /path/to/your.iso`
-   **GUI mode**: `./target/release/rust-system-tools show-gui`

### Test
-   Run tests with: `cargo test`

## Architecture

This project is a Rust tool for mounting ISO files via the UDisks2 D-Bus interface. It has both a command-line interface (CLI) and an optional graphical user interface (GUI).

### Key Modules
-   `src/main.rs`: Main application entry point, handles CLI argument parsing using `clap`.
-   `src/lib.rs`: Core library crate.
-   `src/udisks2.rs`: Contains the logic for interacting with the UDisks2 service over D-Bus using `zbus`. This is where block devices are managed and ISO files are mounted.
-   `src/gui.rs`: Implements the GUI using `eframe` and `egui`. This is an optional feature.
-   `src/config.rs`: Handles loading and saving of the application configuration from a TOML file (`~/.config/rust-system-tools/config.toml`). This is primarily for GUI settings.

### High-level-flow
1.  `main.rs` parses command-line arguments.
2.  If `show-gui` is specified, `gui::run_gui()` is called.
3.  If `mount` is specified, the `udisks2` module's functions are called to perform the mounting operation.
4.  The `udisks2` module communicates with the system's UDisks2 service via D-Bus to perform actions like loop device setup and filesystem mounting.
5.  The `config.rs` module is used by the `gui` module to persist UI settings.
