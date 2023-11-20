use indexed_db_futures::prelude::*;
use serde_wasm_bindgen;
use thiserror::Error;

use wasm_bindgen::JsValue;
use web_sys::DomException;

const DB_AND_STORE_NAME: &str = "_twellik";

#[derive(Debug, Error)]
pub enum IdbError {
    #[error("can't open indexed db: {0}")]
    IdbOpenError(String),
}

impl From<DomException> for IdbError {
    fn from(value: DomException) -> Self {
        let msg = value.to_string().as_string().unwrap_or("".into());
        IdbError::IdbOpenError(msg)
    }
}

impl Into<JsValue> for IdbError {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}
pub async fn open() -> Result<IdbDatabase, IdbError> {
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32(DB_AND_STORE_NAME, 1)?;

    db_req.set_on_upgrade_needed(Some(
        move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if let None = evt
                .db()
                .object_store_names()
                .find(|n| n == DB_AND_STORE_NAME)
            {
                evt.db().create_object_store(DB_AND_STORE_NAME)?;
            }
            Ok(())
        },
    ));

    Ok(db_req.await?)
}

pub async fn put_key(db: &IdbDatabase, key: &str, value: &JsValue) -> Result<(), IdbError> {
    let tx = db.transaction_on_one_with_mode(DB_AND_STORE_NAME, IdbTransactionMode::Readwrite)?;
    let store = tx.object_store(DB_AND_STORE_NAME)?;

    store.put_key_val_owned(key, value)?;
    tx.await.into_result()?;

    Ok(())
}

pub async fn get_key(db: &IdbDatabase, key: &str) -> Result<Option<JsValue>, IdbError> {
    let tx = db.transaction_on_one_with_mode(DB_AND_STORE_NAME, IdbTransactionMode::Readonly)?;
    let store = tx.object_store(DB_AND_STORE_NAME)?;

    let value: Option<JsValue> = store.get_owned(key)?.await?;

    Ok(value)
}

pub async fn keys(db: &IdbDatabase) -> Result<Vec<String>, IdbError> {
    let tx = db.transaction_on_one_with_mode(DB_AND_STORE_NAME, IdbTransactionMode::Readonly)?;
    let store = tx.object_store(DB_AND_STORE_NAME)?;

    let mut names: Vec<String> = Vec::new();
    let js_names = store.get_all_keys()?.await?.to_vec();

    for name in js_names {
        match serde_wasm_bindgen::from_value(name) {
            Ok(r) => names.push(r),
            Err(e) => return Err(IdbError::IdbOpenError(e.to_string())),
        }
    }

    Ok(names)
}
