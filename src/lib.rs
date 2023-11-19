mod cosine;
mod indexed_db;
mod log;

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use indexed_db_futures::IdbDatabase;

use rkyv;
use rkyv::Deserialize;
use serde;
use serde_wasm_bindgen;

use lazy_static::lazy_static; // 1.4.0
use std::sync::Mutex;

lazy_static! {
    // should be Mutex<Collectoin>
    static ref MEM_DB_STATE: Mutex<HashMap<String, Collection>> = Mutex::new(HashMap::new());
}

#[wasm_bindgen]
struct Database {
    db: IdbDatabase,
    collections: HashMap<String, Collection>,
}

#[wasm_bindgen]
impl Database {
    #[wasm_bindgen(constructor)]
    pub async fn new(coll_name: &str) -> Result<Database, JsValue> {
        let db = match indexed_db::open_db(coll_name).await {
            Ok(db) => {
                twellik_debug(format!("Opened db {coll_name}").as_str());
                db
            }
            Err(e) => return Err(e.to_string().into()),
        };

        let mut collections = HashMap::<String, Collection>::new();

        for store_name in db.object_store_names() {
            let collection = read_collection(&store_name).await?;
            collections.insert(store_name, collection);
        }

        Ok(Database { db, collections })
    }
}

#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Clone,
)]
#[archive(check_bytes)]
struct Point {
    /// TODO: id should be uuid or any
    id: String,
    vector: Vec<f64>,
    payload: HashMap<String, String>,
}

/// TODO: Clone is here as a temp workaround,
/// should be removed after implementing Database
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Collection {
    pub points: Vec<Point>,
}

impl Collection {
    pub fn scroll_points(query: &Query) -> Option<Vec<QueryResult>> {
        todo!()
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Query {
    vector: Vec<f64>,
    payload: HashMap<String, String>,
    k: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct QueryResult {
    point: Point,
    distance: f64,
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

fn twellik_log(s: &str) {
    log(format!("Twellik: {s}").as_str())
}

fn twellik_error(s: &str) {
    log_error(format!("Twellik Error: {s}").as_str())
}

fn twellik_warn(s: &str) {
    log_warn(format!("Twellik Warn: {s}").as_str())
}
fn twellik_debug(s: &str) {
    log_debug(format!("Twellik Debug: {s}").as_str())
}

#[wasm_bindgen]
/// create_collection is currently useless, however,
/// we will probably need it in future when we need to set params
/// before inserting items
pub async fn create_collection(name: &str) -> Result<(), JsValue> {
    twellik_warn("HAVE YOU REBUILT WASM?");
    twellik_debug(format!("{name} collection creation invoked.").as_str());

    Ok(())
}

#[wasm_bindgen]
/// TODO: support async
/// TODO: upsert erases all points currently
/// TODO: if it is async, js should use await, right?
pub async fn upsert_points(coll_name: &str, points: JsValue) -> Result<(), JsValue> {
    let db = match indexed_db::open_db(coll_name).await {
        Ok(db) => {
            twellik_debug(format!("Opened db {coll_name}").as_str());
            db
        }
        Err(e) => return Err(e.to_string().into()),
    };

    // TODO: should be async/nonblocking/point-by-point?
    let points: Vec<Point> = serde_wasm_bindgen::from_value(points.clone())?;

    // sync with in-mem state
    let mut mem_db = MEM_DB_STATE.lock().unwrap();

    // erase all points to be consi stent with the current behavior
    let new_coll = Collection { points };

    let b_points = rkyv::to_bytes::<_, 256>(&new_coll.points).unwrap();

    // TODO: check unpacked binary
    // let archived_points = rkyv::check_archived_root::<Vec<Point>>(&b_points[..]).unwrap();
    //let rs_points2: Vec<Point> = archived_points.deserialize(&mut rkyv::Infallible).unwrap();
    let b_points_u8 = b_points.as_slice();
    let b_points_jsval = serde_wasm_bindgen::to_value(&b_points_u8).unwrap();

    match indexed_db::put_key(&db, coll_name, &b_points_jsval).await {
        Ok(_) => {
            twellik_debug(format!("Added points to {coll_name}.").as_str());
        }
        Err(e) => {
            twellik_error(
                format!("Error inserting points to {coll_name}: {}", e.to_string()).as_str(),
            );
        }
    };

    mem_db.insert(coll_name.to_string(), new_coll);

    Ok(())
}

/// Reads collection into memory.
async fn read_collection(coll_name: &str) -> Result<Collection, JsValue> {
    let db = match indexed_db::open_db(coll_name).await {
        Ok(db) => {
            twellik_debug(format!("Opened db {coll_name}").as_str());
            db
        }
        Err(e) => return Err(e.to_string().into()),
    };

    let js_points = match indexed_db::get_key(&db, coll_name).await {
        Ok(v) => match v {
            Some(p) => p,
            None => {
                let msg = format!("Collection {coll_name} is empty.");
                twellik_error(&msg);
                return Err(JsValue::from_str(&msg));
            }
        },
        Err(e) => return Err(e.to_string().into()),
    };

    // TODO: should be async/nonblocking/point-by-point?
    let raw_points: Vec<u8> = serde_wasm_bindgen::from_value(js_points)?;

    let archived_points = rkyv::check_archived_root::<Vec<Point>>(&raw_points[..]).unwrap();
    let points: Vec<Point> = archived_points.deserialize(&mut rkyv::Infallible).unwrap();

    Ok(Collection { points })
}

/// Checks if all fields of `query_fields` are eq to those in `item`
fn match_payload(item: &HashMap<String, String>, query_fields: &HashMap<String, String>) -> bool {
    if query_fields.is_empty() {
        return true;
    }

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
pub async fn scroll_points(coll_name: &str, query: JsValue) -> Result<JsValue, JsValue> {
    let parsed_query: Query = serde_wasm_bindgen::from_value(query)?;

    let coll = read_collection(coll_name).await?;

    let mut matched_points: Vec<QueryResult> = coll
        .points
        .iter()
        .filter(|point| match_payload(&point.payload, &parsed_query.payload))
        .map(|point| {
            let distance = cosine::distance(&parsed_query.vector, &point.vector);
            QueryResult {
                point: point.clone(),
                distance,
            }
        })
        .collect();

    matched_points.sort_by(|a, b| match a.distance.partial_cmp(&b.distance) {
        Some(r) => r,
        None => {
            println!(
                "panic! comparison of these two numbers failed: {0} and {1}",
                &a.distance, &b.distance
            );
            panic!();
        }
    });

    let matched_points: Vec<&QueryResult> = matched_points.iter().take(parsed_query.k).collect();

    twellik_debug(format!("matched: {}", &matched_points.len()).as_str());

    Ok(serde_wasm_bindgen::to_value(&matched_points)?)
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

    #[test]
    fn match_payload_test_two() {
        let item = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
            ("c".to_string(), "three".to_string()),
        ]);

        let query_fields = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "one".to_string()),
        ]);

        let result = match_payload(&item, &query_fields);

        assert!(!result);
    }
}
