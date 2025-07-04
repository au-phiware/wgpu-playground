# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Environment

This project uses **Nix flakes** for reproducible development environments. The `flake.nix` provides:

### Getting Started with Nix
- `nix develop` - Enter the development shell with all dependencies
- The flake provides a complete Rust toolchain with rust-analyzer and rust-src
- Includes all necessary graphics libraries (Vulkan, OpenGL, Wayland, X11)

### Development Tools Included
- `mask` - Task runner (use `mask --help` to see available tasks)
- `cargo-watch` - Watch for file changes and rebuild
- `cargo-edit` - Manage dependencies
- `cargo-audit` - Security auditing
- `wasm-pack` - WebAssembly packaging
- `jujutsu` - Version control
- `gdb`, `valgrind` - Debugging tools

## Development Commands

This project uses a `maskfile.md` for task automation. Common commands:

### Build and Development
- `cargo build` - Build all workspace projects
- `cargo check` - Check all projects without building
- `cargo run --bin hello-wgpu` - Run the hello-wgpu project
- `cargo clean` - Clean build artifacts

### Testing and Quality
- `cargo test` - Run tests for all projects
- `cargo clippy --all-targets --all-features -- -D warnings` - Run linter
- `cargo fmt --all` - Format all code
- `cargo fmt --all -- --check` - Check if code is formatted

### Development Workflow
- `cargo watch -x check -x test -x "clippy --all-targets --all-features -- -D warnings"` - Watch for changes and rebuild
- `cargo audit` - Check dependencies for security vulnerabilities
- `cargo update` - Update dependencies

### WebAssembly
- `wasm-pack build hello-wgpu` - Build the project for WebAssembly

## Project Architecture

This is a Rust workspace containing GPU-accelerated graphics applications using wgpu.

### Structure
- **Root Cargo.toml**: Workspace configuration with shared dependencies
- **hello-wgpu/**: Main graphics application
  - `src/lib.rs`: Entry point with platform-specific initialization
  - `src/app.rs`: Application logic and event handling
  - `src/state.rs`: GPU state management
  - `src/main.rs`: Native binary entry point

### Key Technologies
- **wgpu**: Cross-platform graphics API abstraction
- **winit**: Window creation and event handling
- **WebAssembly**: Dual-target support for native and web platforms

### Platform Support
The application supports both native desktop and WebAssembly targets. The codebase uses conditional compilation (`#[cfg(target_arch = "wasm32")]`) to handle platform-specific code paths.

### Graphics Backend Support
The Nix environment provides comprehensive graphics support:
- **Vulkan**: Full validation layers and loader configuration
- **OpenGL**: Native OpenGL support
- **Wayland**: Modern Linux compositor support
- **X11**: Traditional X Window System support

### Development Notes
- The workspace resolver is set to "3" for the latest cargo features
- Release builds are configured with `strip = true` for smaller binaries
- WebAssembly builds require the `webgl` feature for wgpu compatibility
- Environment variables are pre-configured for Vulkan development