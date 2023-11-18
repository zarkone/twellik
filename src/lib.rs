mod cosine;
mod indexed_db;
mod log;

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use rkyv;
use rkyv::Deserialize;
use serde;
use serde_wasm_bindgen;

use lazy_static::lazy_static; // 1.4.0
use std::sync::Mutex;

lazy_static! {
    // should be Mutex<Collectoin>
    static ref MEM_DB_STATE: Mutex<Vec<Point>> = Mutex::new(vec![]);
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Collection {
    pub points: Vec<Point>,
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
}

fn twellik_log(s: &str) {
    log(format!("Twellik: {s}").as_str())
}

fn twellik_error(s: &str) {
    log_error(format!("Twellik Error: {s}").as_str())
}

#[wasm_bindgen]
/// create_collection is currently useless, however,
/// we will probably need it in future when we need to set params
/// before inserting items
pub async fn create_collection(name: &str) -> Result<(), JsValue> {
    twellik_log("HAVE YOU REBUILT WASM?");
    twellik_log(format!("{name} collection creation invoked.").as_str());

    Ok(())
}

#[wasm_bindgen]
/// TODO: support async
/// TODO: upsert erases all points currently
/// TODO: if it is async, js should use await, right?
pub async fn upsert_points(coll_name: &str, points: JsValue) -> Result<(), JsValue> {
    let db = match indexed_db::open_db(coll_name).await {
        Ok(db) => {
            twellik_log(format!("Opened db {coll_name}").as_str());
            db
        }
        Err(e) => return Err(e.to_string().into()),
    };

    // TODO: should be async/nonblocking/point-by-point?
    let rs_points: Vec<Point> = serde_wasm_bindgen::from_value(points.clone())?;

    // sync with in-mem state
    let mut mem_db = MEM_DB_STATE.lock().unwrap();

    // erase all points to be consistent with the current behavior
    *mem_db = Vec::new();
    for point in &rs_points {
        mem_db.push(point.clone())
    }

    let b_points = rkyv::to_bytes::<_, 256>(&rs_points).unwrap();

    // TODO: check unpacked binary
    // let archived_points = rkyv::check_archived_root::<Vec<Point>>(&b_points[..]).unwrap();
    //let rs_points2: Vec<Point> = archived_points.deserialize(&mut rkyv::Infallible).unwrap();
    let b_points_u8 = b_points.as_slice();
    let b_points_jsval = serde_wasm_bindgen::to_value(&b_points_u8).unwrap();

    match indexed_db::put_key(&db, coll_name, &b_points_jsval).await {
        Ok(_) => {
            twellik_log(format!("Added points to {coll_name}.").as_str());
        }
        Err(e) => {
            twellik_log(
                format!("Error inserting points to {coll_name}: {}", e.to_string()).as_str(),
            );
        }
    };

    Ok(())
}

/// Reads collection into memory.
async fn read_collection(coll_name: &str) -> Result<Collection, JsValue> {
    let mut mem_db = MEM_DB_STATE.lock().unwrap();
    if mem_db.is_empty() {
        let db = match indexed_db::open_db(coll_name).await {
            Ok(db) => {
                twellik_log(format!("Opened db {coll_name}").as_str());
                db
            }
            Err(e) => return Err(e.to_string().into()),
        };

        let js_points = match indexed_db::get_key(&db, coll_name).await {
            Ok(v) => match v {
                Some(p) => p,
                None => {
                    let msg = format!("Collecton {coll_name} is empty.");
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
    } else {
        twellik_log("skipping db read, return collection from mem");
        Ok(Collection {
            // here we clone a collection, so no point of keeping points in mem ATM actually
            // however, it should be fixed when we move to keeping Collection
            points: mem_db.to_vec(),
        })
    }
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
    twellik_log("start parsing.");

    let parsed_query: Query = serde_wasm_bindgen::from_value(query)?;
    twellik_log(format!("query: {:?}", &parsed_query).as_str());

    let coll = read_collection(coll_name).await?;
    twellik_log(format!("coll: {:?}", &coll).as_str());

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

    twellik_log(format!("matched: {:?}", &matched_points).as_str());

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
