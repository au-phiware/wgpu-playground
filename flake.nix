{
  description = "WebGPU playground with Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            
            # Graphics and windowing libraries for wgpu
            libxkbcommon
            libGL
            
            # Wayland support
            wayland
            
            # X11 support
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            
            # Vulkan support
            vulkan-loader
            vulkan-headers
            vulkan-validation-layers
            
            # Mesa drivers for OpenGL/Vulkan
            mesa
            
            # OpenSSL for reqwest and networking
            openssl
            openssl.dev
            
            # Additional development tools
            cargo-watch
            cargo-edit
            cargo-audit
            mask
            jujutsu
            wasm-pack
            miniserve
            claude-code
            mprocs
            
            # WebGPU-enabled Chromium script
            (pkgs.writeShellScriptBin "chromium-webgpu" ''
              exec chromium --enable-features=WebGPU,Vulkan --enable-unsafe-webgpu --disable-dawn-features=disallow_unsafe_apis "$@"
            '')
            
            # Optional: debugging and profiling
            gdb
            valgrind
          ];
          
          shellHook = ''
            export RUST_BACKTRACE=1
            
            # Set up Mesa drivers
            export LIBGL_DRIVERS_PATH="${pkgs.mesa}/lib/dri"
            export VK_ICD_FILENAMES="${pkgs.mesa}/share/vulkan/icd.d/radeon_icd.x86_64.json:${pkgs.mesa}/share/vulkan/icd.d/intel_icd.x86_64.json"

            
            echo "🦀 Rust WebGPU development environment loaded!"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "Chromium available for WebGPU testing: $(chromium --version 2>/dev/null || echo 'Not found')"
            echo "🌐 Use 'chromium-webgpu http://127.0.0.1:8080' to test WebGPU apps"
          '';
          
          # Set environment variables for graphics libraries
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libGL
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.vulkan-loader
            pkgs.openssl
          ];
        };
      }
    );
}
