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
- **gol/**: Conway's Game of Life implementation
  - `src/lib.rs`: Entry point with platform-specific initialization
  - `src/app.rs`: Application logic and event handling
  - `src/gpu.rs`: GPU context and surface management
  - `src/renderer.rs`: Rendering pipeline and display logic
  - `src/conway.rs`: Conway's Game of Life compute shader implementation
  - `src/conway.wgsl`: WGSL compute shader for Conway's rules
  - `src/display.wgsl`: WGSL display shader for rendering
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

## WebGPU Development Workflow

### Browser Setup
- Use `chromium-webgpu` command (provided by flake) which launches Chromium with WebGPU flags
- Required flags: `--enable-features=WebGPU,Vulkan --enable-unsafe-webgpu --disable-dawn-features=disallow_unsafe_apis`
- Test at chrome://gpu to verify WebGPU is enabled

### Development Commands
- `mask start gol` - Start development server with auto-rebuild, serve, and browser launch
- `mask pack gol` - Build WebAssembly package
- `mask serve gol` - Serve the application locally
- Uses `mprocs` to run multiple processes concurrently

### Cross-Platform Considerations
- **Storage Buffers vs Textures**: Use storage textures for better WebGPU compatibility
- **Backend Selection**: Use `wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL` for web
- **Features**: Add web-sys features like "Location" for browser-specific functionality

## wgpu Surface Lifetime Management

### The Arc<Window> Solution
When working with wgpu surfaces, use `Arc<Window>` to solve lifetime issues:

```rust
// ✅ Correct approach
let window = Arc::new(event_loop.create_window(attributes)?);
let surface = instance.create_surface(window.clone())?; // Creates Surface<'static>

// ❌ Problematic approach
let surface = instance.create_surface(&window)?; // Creates Surface<'window>
```

### Why Arc<Window> Works
- `Arc<Window>` has `'static` lifetime (owned, not borrowed)
- `Surface<'static>` can be stored in structs without lifetime parameters
- Multiple components can share ownership of the window
- The surface creation takes ownership of an Arc clone, not a borrow

### Architecture Pattern
```rust
pub struct SurfaceManager {
    window: Arc<Window>,           // Shared ownership
    surface: Surface<'static>,     // Static lifetime
    config: SurfaceConfiguration,
    is_configured: bool,
}
```

## Keyboard Event Handling

### winit Key Patterns
Use string literals for key matching (no allocations):

```rust
match event.logical_key {
    Key::Character(ref key) if key == "r" => {
        // Handle R key
    }
    Key::Character(ref key) if key == "q" => {
        // Handle Q key
    }
    _ => {}
}
```

### Platform-Specific Actions
```rust
#[cfg(target_arch = "wasm32")]
{
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}
#[cfg(not(target_arch = "wasm32"))]
{
    event_loop.exit();
}
```

### Modifier Keys
Track modifier state separately:
```rust
WindowEvent::ModifiersChanged(modifiers) => {
    self.keyboard_modifiers = modifiers.state();
}
```

## Version Control with Jujutsu

### Editing Commit History
- **Mutable commits**: `jj describe <commit-id>`
- **Immutable commits**: Use `jj split` to separate files you want to keep
- **Split interaction**: Use spacebar to select files, Enter to confirm

### Splitting Commits
```bash
jj edit <commit-id>
jj split
# Use spacebar to select files to keep in original commit
# Press Enter to confirm
```

### Pushing Changes
```bash
jj bookmark set main -r @    # Move main bookmark to current commit
jj git push --force          # Force push to update remote
```

### Common Workflow
1. Make changes
2. `jj describe` to set commit message
3. `jj bookmark set main -r @` to update main bookmark
4. `jj git push --force` to push changes

## Conway's Game of Life Implementation

### Compute Shader Architecture
- **Ping-pong textures**: Two textures alternating as input/output
- **Storage textures**: Use `R32Float` format for cell states
- **Workgroup size**: 16x16 for optimal GPU utilization

### Initialization
```rust
let initial_state: Vec<f32> = (0..GRID_SIZE * GRID_SIZE)
    .map(|_| if rand::random::<f32>() > 0.7 { 1.0 } else { 0.0 })
    .collect();
```

### Cross-Platform Random
- Use `getrandom` with `wasm_js` feature for WASM compatibility
- Add to Cargo.toml: `getrandom = { workspace = true, features = ["wasm_js"] }`