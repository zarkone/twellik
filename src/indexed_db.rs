use indexed_db_futures::prelude::*;
use serde_wasm_bindgen;
use thiserror::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::DomException;

#[derive(Debug, Error)]
pub enum IdbError {
    #[error("can't open indexed db: {0}")]
    IdbOpenError(String),

    #[error("can't WASM env error: {0}")]
    EnvError(String),
}
impl From<DomException> for IdbError {
    fn from(value: DomException) -> Self {
        log_error(value.to_string().as_string().unwrap_or("".into()).as_str());
        IdbError::IdbOpenError(value.as_string().unwrap_or("".into()))
    }
}

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

pub async fn put_key(key: &str, value: &JsValue) -> Result<(), IdbError> {
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32("_twellik", 1)?;

    db_req.set_on_upgrade_needed(Some(
        move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if let None = evt.db().object_store_names().find(|n| n == "_twellik") {
                evt.db().create_object_store("_twellik")?;
            }
            Ok(())
        },
    ));

    let db: IdbDatabase = db_req.await?;

    let tx: IdbTransaction =
        db.transaction_on_one_with_mode("_twellik", IdbTransactionMode::Readwrite)?;
    let store: IdbObjectStore = tx.object_store("_twellik")?;

    store.put_key_val_owned(key, value)?;
    tx.await.into_result()?;

    Ok(())
}
pub async fn open() -> Result<IdbDatabase, IdbError> {
    let db_req: OpenDbRequest = IdbDatabase::open_u32("_twellik", 1)?;
    let db: IdbDatabase = db_req.await?;
    Ok(db)
}

pub async fn get_key(key: &str) -> Result<Option<JsValue>, IdbError> {
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32("_twellik", 1)?;

    db_req.set_on_upgrade_needed(Some(
        move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if let None = evt.db().object_store_names().find(|n| n == "_twellik") {
                evt.db().create_object_store("_twellik")?;
            }
            Ok(())
        },
    ));

    let db: IdbDatabase = db_req.await?;

    let tx: IdbTransaction =
        db.transaction_on_one_with_mode("_twellik", IdbTransactionMode::Readonly)?;
    let store: IdbObjectStore = tx.object_store("_twellik")?;

    let value: Option<JsValue> = store.get_owned(key)?.await?;

    Ok(value)
}

pub async fn keys() -> Result<Vec<String>, IdbError> {
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32("_twellik", 1)?;

    db_req.set_on_upgrade_needed(Some(
        move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if let None = evt.db().object_store_names().find(|n| n == "_twellik") {
                evt.db().create_object_store("_twellik")?;
            }
            Ok(())
        },
    ));

    let db: IdbDatabase = db_req.await?;

    let tx: IdbTransaction =
        db.transaction_on_one_with_mode("_twellik", IdbTransactionMode::Readonly)?;
    let store: IdbObjectStore = tx.object_store("_twellik")?;

    let js_names = store.get_all_keys()?.await?.to_vec();
    let mut names: Vec<String> = Vec::new();
    for name in js_names {
        match serde_wasm_bindgen::from_value(name) {
            Ok(r) => names.push(r),
            Err(e) => return Err(IdbError::IdbOpenError(e.to_string())),
        }
    }

    Ok(names)
}
