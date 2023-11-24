mod cosine;
mod dist;
mod indexed_db;
mod log;
mod point;

extern crate console_error_panic_hook;

use indexed_db_futures::IdbDatabase;
use point::Point;
use rkyv;
use rkyv::{Archive, Deserialize, Serialize};
use serde;
use serde_wasm_bindgen;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use hnsw::{Hnsw, Searcher};
use rand_pcg::Pcg64;

// TODO: remove
fn test_hnsw() -> (Hnsw<dist::Cosine, Vec<f64>, Pcg64, 12, 24>, Searcher<u64>) {
    let mut searcher = Searcher::default();
    let mut hnsw = Hnsw::new(dist::Cosine);

    let features = vec![
        vec![0.0, 0.0, 0.0, 1.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 1.0, 1.0, 0.0],
        vec![1.0, 1.0, 0.0, 0.0],
        vec![1.0, 0.0, 0.0, 1.0],
    ];

    for feature in &features {
        hnsw.insert(feature.clone(), &mut searcher);
    }
    log::debug(format!("hnsw len: {}", hnsw.len()).as_str());
    (hnsw, searcher)
}

#[wasm_bindgen]
pub struct Twellik {
    db: IdbDatabase,
    collections: HashMap<String, Collection>,
}

#[wasm_bindgen]
impl Twellik {
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<Twellik, JsValue> {
        console_error_panic_hook::set_once();
        log::warn("have you updated your WASM?");

        // TODO: remove
        test_hnsw();

        let db = indexed_db::open()
            .await
            .map_err(<indexed_db::IdbError as Into<JsValue>>::into)?;
        let collections = Twellik::pull_db(&db).await?;

        log::debug("initialized db.");
        Ok(Twellik { collections, db })
    }

    async fn pull_db(db: &IdbDatabase) -> Result<HashMap<String, Collection>, JsValue> {
        let mut collections = HashMap::<String, Collection>::new();

        let key_names = match indexed_db::keys(&db).await {
            Ok(sn) => sn,
            Err(e) => return Err(e.to_string().into()),
        };

        for key_name in key_names {
            let js_coll = match indexed_db::get_key(&db, &key_name).await {
                Ok(v) => match v {
                    Some(p) => p,
                    None => {
                        let msg = format!("Collection {key_name} is empty.");
                        log::error(&msg);
                        return Err(JsValue::from_str(&msg));
                    }
                },
                Err(e) => return Err(e.to_string().into()),
            };

            // TODO: should be async/nonblocking/point-by-point?
            let raw_coll: &[u8] = serde_wasm_bindgen::from_value(js_coll)?;

            let archived_coll = match rkyv::check_archived_root::<Collection>(raw_coll) {
                Ok(r) => r,
                Err(e) => {
                    log::error("pull_db: error checking bytes of db value -- did you change the data or datastructure?");
                    return Err(e.to_string().into());
                }
            };
            let coll: Collection = match archived_coll.deserialize(&mut rkyv::Infallible) {
                Ok(r) => r,
                Err(e) => {
                    log::error("pull_db: while trying to deserialize:");
                    return Err(e.to_string().into());
                }
            };

            collections.insert(key_name, coll);
        }

        Ok(collections)
    }
    #[wasm_bindgen]
    /// TODO: quadratic complexity here for now,
    /// to be addressed after HNSW impl
    pub async fn upsert_points(&mut self, coll_name: &str, points: JsValue) -> Result<(), JsValue> {
        let new_points: Vec<Point> = serde_wasm_bindgen::from_value(points.clone())?;

        if let Some(coll) = self.collections.get_mut(coll_name) {
            for new_point in new_points {
                // TODO: check what if insert the same
                coll.index.insert(new_point)
            }
            log::debug("collection updated.");
        } else {
            let name = coll_name.to_string();
            let mut hnsw = Hnsw::new(dist::Cosine);
            let coll = Collection {
                index: hnsw,

                name: name.clone(),
            };
            log::debug("new collection created.");
            self.collections.insert(name, coll);
        };

        self.serialize_collection(coll_name).await?;

        Ok(())
    }

    async fn serialize_collection(&self, coll_name: &str) -> Result<(), JsValue> {
        let coll = match self.collections.get(coll_name) {
            Some(c) => c,
            None => {
                let msg = format!(
                    "FATAL: failed to serialize {coll_name}: collection not found in memory."
                );
                log::error(&msg);
                return Err(msg.into());
            }
        };
        let b_coll = rkyv::to_bytes::<_, 256>(&coll).unwrap();
        let b_coll_u8 = b_coll.as_slice();
        let b_coll_jsval = serde_wasm_bindgen::to_value(&b_coll_u8).unwrap();

        log::debug(format!("Writing collection {} to IndexedDB", &coll.name).as_str());

        match indexed_db::put_key(&self.db, &coll.name, &b_coll_jsval).await {
            Ok(_) => {
                log::debug(format!("Added points to {}.", &coll.name).as_str());
                Ok(())
            }
            Err(e) => {
                let msg = format!(
                    "Error inserting points to {}: {}",
                    &coll.name,
                    e.to_string()
                );
                log::error(&msg);
                Err(msg.into())
            }
        }
    }

    #[wasm_bindgen]
    pub async fn scroll_points(&self, coll_name: &str, query: JsValue) -> Result<JsValue, JsValue> {
        let parsed_query: Query = serde_wasm_bindgen::from_value(query)?;
        let query_point = Point {
            id: 0,
            vector: parsed_query.vector.clone(),
            payload: parsed_query.payload.clone(),
        };

        let coll = match self.collections.get(coll_name) {
            Some(c) => c,
            None => {
                let msg = format!(
                    "FATAL: failed to serialize {coll_name}: collection not found in memory."
                );
                log::error(&msg);
                return Err(msg.into());
            }
        };

        let mut matched_points: Vec<QueryResult> = coll
            .index
            .nearest(query_point)
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

        let matched_points: Vec<&QueryResult> =
            matched_points.iter().take(parsed_query.k).collect();

        log::debug(format!("matched: {}", &matched_points.len()).as_str());

        Ok(serde_wasm_bindgen::to_value(&matched_points)?)
    }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
struct Collection {
    pub name: String,
    pub index: Hnsw<dist::Cosine, Point, Pcg64, 12, 24>,
}

impl Collection {}
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
