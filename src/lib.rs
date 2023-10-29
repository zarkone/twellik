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

    #[wasm_bindgen(js_namespace = localStorage, js_name = getItem)]
    fn local_storage_get_item(key: &str, val: &str);

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
/// TODO: support async
pub fn upsert_points(coll_name: &str, points: JsValue) -> Result<(), JsValue> {
    // TODO: probably can just passthrough
    let rs_points: Vec<Point> = serde_wasm_bindgen::from_value(points.clone())?;
    let js_points = match serde_json::to_string(&rs_points) {
        Ok(p) => p,
        Err(e) => e.to_string(),
    };
    let local_storage_name = make_local_storage_collection_name(&coll_name);

    twellik_log(format!("upserting points in {local_storage_name}: {:?}", &rs_points).as_str());
    local_storage_set_item(&local_storage_name, &js_points);

    Ok(())
}

/// Reads collection into memory.
fn read_collection(coll_name: &str) -> Collection {
    let local_storage_name = make_local_storage_collection_name(&coll_name);
    todo!()
}

/// Checks if all fields of `query_fields` are eq to those in `item`
fn match_payload(item: &HashMap<String, String>, query_fields: &HashMap<String, String>) -> bool {
    for (key, val) in query_fields {
        let item_val = item.get(key);
        if let Some(found_key) = item_val {
            if found_key.eq(val) {
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

#[wasm_bindgen]
/// Searches through points and returns K amount of closest points
/// which match the query
/// TODO: support async
pub fn scroll_points(coll_name: &str, query: &str) {
    // query.vector: [... f32]
    // query.payload: { ... }
}

#[cfg(test)]
mod tests {
    use crate::match_payload;
    use std::collections::HashMap;

    #[test]
    fn match_payload_test_happy() {
        let item = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
            ("c".to_string(), "three".to_string()),
        ]);

        let query_fields = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
            ("c".to_string(), "three".to_string()),
        ]);

        let result = match_payload(&item, &query_fields);

        assert!(result);
    }
}
