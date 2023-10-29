# Twellik

Sample WASM application, which supposed to become a simple vector db for browser

## Building

You need to have `wasm-pack` installed.

```sh
make
```

This will build wasm ready for browser into `pkg` folder.

Put one of the `examples` into `pkg` and run `python -m http.server 8008`

WASM module logs with `Twellik` prefix
