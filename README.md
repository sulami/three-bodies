# Three Bodies

This is a simple three body simulation, built over a lunch break to try
out [macroquad](https://docs.rs/macroquad/latest/macroquad/index.html).

https://github.com/sulami/three-bodies/assets/1843193/d8a41847-a475-46c8-8eb9-396d64411175

A known issue right now is that the bodies wrap around the viewport, but gravity does not, which leads to weird behaviour near the edges.

## Building

For the native version, run

```sh
cargo run --release
```

For the wasm version, run

```sh
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/three-bodies.wasm docs/
```
