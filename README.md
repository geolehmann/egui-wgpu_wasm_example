# egui-wgpu_wasm_example
This is a minimal example how to use the egui-wgpu crate for a wasm32 target. Based on https://github.com/zhouhang95/egui-winit-wgpu and https://github.com/Twinklebear/wgpu-rs-test.

Usage:
cargo build --target wasm32-unknown-unknown
wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/wgpu-test.wasm

Then open the generated html file in Chrome Canary.
