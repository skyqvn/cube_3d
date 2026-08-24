# cube_3d

A 3D cube rendered with six colored faces, built with **Rust** + **wgpu** + **winit**. Runs natively on Windows/Linux and in the browser via WebAssembly.

![screenshot](screenshot.png)

## Features

- Six textured cube faces with colored backgrounds and text labels
- Mouse drag to rotate the cube
- Mouse wheel to zoom in/out
- Consistent visual size across window sizes
- FPS counter in the window title
- Cross-platform: desktop (Vulkan/Metal/DX12) and browser (WebGPU)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, edition 2021)
- For WASM: [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) CLI

```bash
cargo install wasm-bindgen-cli
```

## Build

### Desktop

```bash
cargo build --release
cargo run --release
```

### WASM (Browser)

```bash
# Build WASM and generate JS glue
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/cube_3d.wasm --out-dir www/pkg --target web

# Or use the batch script (Windows)
build_wasm.bat

# Serve locally
python -m http.server 8000 -d www
```

Then open `http://127.0.0.1:8000` in a WebGPU-enabled browser (Chrome 113+, Edge 113+).

## Controls

| Action       | Input                |
| ------------ | -------------------- |
| Rotate       | Left-click drag      |
| Zoom         | Mouse wheel / scroll |
| Release grab | <kbd>Esc</kbd>       |

## Project Structure

```
cube_3d/
├── assets/
│   ├── shader.wgsl         # WGSL vertex/fragment shader
│   └── font_8x8.bin        # 8×8 bitmap font for face labels
├── src/
│   ├── main.rs             # Entry point, event loop, input handling
│   ├── state.rs            # WGPU state, pipeline, rendering
│   ├── texture.rs          # Procedural texture atlas generation
│   └── vertex.rs           # Cube geometry (vertices & indices)
├── www/
│   ├── index.html          # WASM host page
│   └── pkg/                # Generated WASM + JS glue
├── Cargo.toml
└── README.md
```

## Tech Stack

| Component               | Crate                                                 |
| ----------------------- | ----------------------------------------------------- |
| Window & events         | [winit](https://crates.io/crates/winit)               |
| GPU rendering           | [wgpu](https://crates.io/crates/wgpu)                 |
| Math (MVP, quaternions) | [glam](https://crates.io/crates/glam)                 |
| WASM bindings           | [wasm-bindgen](https://crates.io/crates/wasm-bindgen) |

## License

[MIT](https://opensource.org/licenses/MIT)
