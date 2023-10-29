use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use serde_json;
use serde_wasm_bindgen;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    id: String,
    vector: Vec<f64>,
    payload: HashMap<String, String>,
}

struct Collection {
    points: Vec<Point>,
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = localStorage, js_name = setItem)]
    fn local_storage_set_item(key: &str, val: &str);
}

fn twellik_log(s: &str) {
    log(format!("Twellik: {s}").as_str())
}
fn make_local_storage_collection_name(name: &str) -> String {
    let prefix = "__twellik";
    format!("{prefix}_{name}")
}

#[wasm_bindgen]
pub fn create_collection(name: &str) -> Result<(), JsValue> {
    let local_storage_name = make_local_storage_collection_name(&name);

    local_storage_set_item(&local_storage_name, &"");
    twellik_log(format!("{local_storage_name} collection created.").as_str());
    Ok(())
}

// TODO: how to pass types from/to JS land:
// https://rustwasm.github.io/wasm-bindgen/reference/arbitrary-data-with-serde.html?highlight=array#javascript-usage

#[wasm_bindgen]
pub fn upsert_points(coll_name: &str, points: JsValue) -> Result<(), JsValue> {
    // TODO: probably can just passthrough
    let rs_points: Vec<Point> = serde_wasm_bindgen::from_value(points.clone())?;
    let js_points = match serde_json::to_string(&rs_points) {
        Ok(p) => p,
        Err(e) => e.to_string(),
    };
    let coll_name_prefixed = make_local_storage_collection_name(&coll_name);

    twellik_log(format!("uprerting points in {coll_name}: {:?}", &rs_points).as_str());
    local_storage_set_item(&coll_name_prefixed, &js_points);
    Ok(())
}

/// Reads collection into memory.
fn read_collection(name: &str) -> Collection {
    todo!()
}

#[wasm_bindgen]
pub fn scroll_points(query: &str) {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 3;
        assert_eq!(result, 4);
    }
}
