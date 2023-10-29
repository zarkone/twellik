# Twellik

Sample WASM application, which supposed to become a simple vector db for browser

## Example

```js
      import init, { create_collection, upsert_points, scroll_points } from "./twellik.js";

      init().then( () => {
          let coll = "test_collection"
          create_collection(coll);
          let points = [
              {"id": "1", "vector": [0.05, 0.61, 0.76, 0.74], "payload": {"text": "text1"}},
              {"id": "2", "vector": [0.19, 0.81, 0.75, 0.11], "payload": {"text": "text4"}},
              {"id": "3", "vector": [0.36, 0.55, 0.47, 0.94], "payload": {"text": "text2"}},
              {"id": "4", "vector": [0.18, 0.01, 0.85, 0.80], "payload": {"text": "text4"}}
          ]
          upsert_points(coll, points)

          let query = {
              vector: [0.05, 0.61, 0.76, 0.72],
              payload: { text: "text4" },
              k: 10
          }

          let searchRes = scroll_points(coll, query)
          console.log(searchRes)

          let emtpyPayloadQuery = {
              vector: [0.05, 0.41, 0.26, 0.12],
              payload: {},
              k: 3
          }

          let emptyQueryRes = scroll_points(coll, emtpyPayloadQuery)
          console.log(emptyQueryRes)

     });

```

## Build

You need to have `wasm-pack` installed.

```sh
make
```

This will build wasm ready for browser into `pkg` folder.

Put one of the `examples` into `pkg` and run `python -m http.server 8008`

WASM module logs with `Twellik` prefix
