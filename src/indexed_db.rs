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
async fn open_db(name: &str) -> Result<(), IdbError> {
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32(name, 1)?;
    db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
        // Check if the object store exists; create it if it doesn't
        if let None = evt.db().object_store_names().find(|n| n == "my_store") {
            evt.db().create_object_store("my_store")?;
        }
        Ok(())
    }));

    let db: IdbDatabase = db_req.await?;

    // Insert/overwrite a record
    let tx: IdbTransaction =
        db.transaction_on_one_with_mode("my_store", IdbTransactionMode::Readwrite)?;
    let store: IdbObjectStore = tx.object_store("my_store")?;

    let value_to_put: JsValue = JsValue::from_str("hello");
    store.put_key_val_owned("my_key", &value_to_put)?;

    // IDBTransactions can have an Error or an Abort event; into_result() turns both into a
    // DOMException
    tx.await.into_result()?;
    Ok(())
}
