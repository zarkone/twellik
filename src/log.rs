use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn log_error(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn log_warn(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = debug)]
    fn log_debug(s: &str);
}

pub fn _info(s: &str) {
    log(format!("Twellik: {s}").as_str())
}

pub fn error(s: &str) {
    log_error(format!("Twellik Error: {s}").as_str())
}

pub fn _warn(s: &str) {
    log_warn(format!("Twellik Warn: {s}").as_str())
}
pub fn debug(s: &str) {
    log_debug(format!("Twellik Debug: {s}").as_str())
}
