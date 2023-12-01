mod collection;
mod dist;
mod index;
mod indexed_db;
mod log;
mod point;
mod query;

extern crate console_error_panic_hook;

use crate::collection::{Collection, PointId};
use crate::index::{HnswIndex, Index};
use crate::point::Point;
use crate::query::{Query, QueryResult};
use indexed_db_futures::IdbDatabase;
use rkyv;
use rkyv::Deserialize;
use serde_wasm_bindgen;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Twellik {
    db: IdbDatabase,
    collections: HashMap<String, Collection>,
}

#[wasm_bindgen]
impl Twellik {
    pub async fn close_db(&self) {
        self.db.close()
    }
    #[wasm_bindgen(constructor)]
    pub async fn new() -> Result<Twellik, JsValue> {
        console_error_panic_hook::set_once();
        log::warn("have you updated your WASM?");

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
            let js_points = match indexed_db::get_key(&db, &key_name).await {
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

            let raw_points: Vec<u8> = serde_wasm_bindgen::from_value(js_points)?;

            let archived_points = match rkyv::check_archived_root::<HashMap<PointId, Point>>(
                &raw_points,
            ) {
                Ok(r) => r,
                Err(e) => {
                    log::error("pull_db: error checking bytes of db value -- did you change the data or datastructure?");
                    return Err(e.to_string().into());
                }
            };
            let points: HashMap<PointId, Point> =
                match archived_points.deserialize(&mut rkyv::Infallible) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error("pull_db: while trying to deserialize:");
                        return Err(e.to_string().into());
                    }
                };

            let mut hnsw = HnswIndex::default();

            for (_id, point) in &points {
                hnsw.insert(point.clone());
            }

            let coll = Collection {
                name: key_name.clone(),
                points,
                index: Box::new(hnsw),
            };

            collections.insert(key_name, coll);
        }

        Ok(collections)
    }
    #[wasm_bindgen]
    pub async fn upsert_points(&mut self, coll_name: &str, points: JsValue) -> Result<(), JsValue> {
        let new_points: Vec<Point> = serde_wasm_bindgen::from_value(points.clone())?;

        if let Some(coll) = self.collections.get_mut(coll_name) {
            for point in new_points {
                if None == coll.points.get(&point.id) {
                    coll.points.insert(point.id.clone(), point.clone());
                    coll.index.insert(point)
                }
            }
            log::debug("collection updated.");
        } else {
            let name = coll_name.to_string();
            let mut hnsw = HnswIndex::default();
            let mut points: HashMap<PointId, Point> = HashMap::new();
            for point in new_points {
                points.insert(point.id.clone(), point.clone());
                hnsw.insert(point);
            }
            let coll = Collection {
                index: Box::new(hnsw),
                points,
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
        let b_points = rkyv::to_bytes::<_, 256>(&coll.points).unwrap();
        let b_points_u8 = b_points.as_slice();
        let b_points_jsval = serde_wasm_bindgen::to_value(&b_points_u8).unwrap();

        log::debug(format!("Writing collection {} to IndexedDB", &coll.name).as_str());

        match indexed_db::put_key(&self.db, &coll.name, &b_points_jsval).await {
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
    pub async fn scroll_points(
        &mut self,
        coll_name: &str,
        query: JsValue,
    ) -> Result<JsValue, JsValue> {
        let parsed_query: Query = serde_wasm_bindgen::from_value(query)?;

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

        let mut matched_points: Vec<QueryResult> = coll.index.scroll(&parsed_query);

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
