# egui-wgpu_wasm_example
This is a minimal example how to use the egui-wgpu crate for a wasm32-unknown-unknown target. Based on https://github.com/zhouhang95/egui-winit-wgpu and https://github.com/Twinklebear/wgpu-rs-test.

Prerequisites:
```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo update -p wasm-bindgen
```
Also be sure to set 
```
[target.wasm32-unknown-unknown]
rustflags = "--cfg=web_sys_unstable_apis"
```
in your config.toml

Usage:
```
cargo build --target wasm32-unknown-unknown  
wasm-bindgen --out-dir target/generated/ --web target/wasm32-unknown-unknown/debug/wgpu-test.wasm  
```
Then you can open the generated html file in Chrome or Opera (as of December 2023, Firefox does not support WebGPU without a special flag).
