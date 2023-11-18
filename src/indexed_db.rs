use crate::log;
use indexed_db_futures::prelude::*;
use thiserror::Error;
use wasm_bindgen::JsValue;
use web_sys::DomException;

#[derive(Debug, Error)]
pub enum IdbError {
    #[error("can't opend indexed db: {0}")]
    IdbOpenError(String),

    #[error("can't WASM env error: {0}")]
    EnvError(String),
}
impl From<DomException> for IdbError {
    fn from(value: DomException) -> Self {
        IdbError::IdbOpenError(value.as_string().unwrap_or("".into()))
    }
}

pub async fn open_db(name: &str) -> Result<IdbDatabase, IdbError> {
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32("_twellik", 1)?;
    let cloned_name = name.to_string();
    db_req.set_on_upgrade_needed(Some(
        move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Check if the object store exists; create it if it doesn't
            if let None = evt.db().object_store_names().find(|n| n == &cloned_name) {
                evt.db().create_object_store(&cloned_name)?;
            }
            Ok(())
        },
    ));

    let db: IdbDatabase = db_req.await?;

    Ok(db)
}

pub async fn put_key(db: &IdbDatabase, key: &str, value: &JsValue) -> Result<(), IdbError> {
    let tx: IdbTransaction = db.transaction_on_one_with_mode(key, IdbTransactionMode::Readwrite)?;
    let store: IdbObjectStore = tx.object_store(key)?;

    store.put_key_val_owned(key, value)?;
    tx.await.into_result()?;

    Ok(())
}
