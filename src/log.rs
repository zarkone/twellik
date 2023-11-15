use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log(s: &str);

}

pub fn twellik_log(s: &str) {
    log(format!("Twellik: {s}").as_str())
}
