use wasm_bindgen::prelude::*;

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
pub fn create_collection(name: &str) {
    let local_storage_name = make_local_storage_collection_name(&name);
    local_storage_set_item(&local_storage_name, &"");
    twellik_log(format!("{local_storage_name} collection created.").as_str())
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello, {}!", name));
}
