use crate::log;
use core::future::Future;
use core::pin::Pin;
use core::task;
use core::task::Poll;
use js_sys::Function;
use thiserror::Error;
use wasm_bindgen::prelude::{Closure, JsCast};
use web_sys::IdbOpenDbRequest;
use web_sys::IdbRequestReadyState;

#[derive(Debug, Error)]
pub enum IdbError {
    #[error("can't opend indexed db: {0}")]
    IdbOpenError(String),

    #[error("IdbFactory error: {0}")]
    IdbFactoryError(String),

    #[error("can't WASM env error: {0}")]
    EnvError(String),
}

pub struct IndexedDB<'a> {
    pub name: &'a str,
    open_db_request: IdbOpenDbRequest,
}

impl IndexedDB<'_> {
    pub fn new(name: &str, open_db_request: IdbOpenDbRequest) -> IndexedDB {
        IndexedDB {
            name,
            open_db_request,
        }
    }
}

impl<'a> Future for IndexedDB<'a> {
    type Output = IdbRequestReadyState;
    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> task::Poll<Self::Output> {
        match self.open_db_request.ready_state() {
            IdbRequestReadyState::Pending => {
                let waker = ctx.waker();

                let cb = Closure::<dyn Fn()>::new(move || {
                    waker.wake();
                });

                let r = match JsCast::dyn_ref::<Function>(cb.as_ref()) {
                    Some(f) => f,
                    None => {
                        log::log("[indexed_db::poll] can't cast cb ");
                        return Poll::Ready(IdbRequestReadyState::Done);
                    }
                };

                self.open_db_request.set_onsuccess(Some(r));

                Poll::Pending
            }
            IdbRequestReadyState::Done => Poll::Ready(IdbRequestReadyState::Done),
        }
    }
}
async fn open_db(name: &str) -> Result<(), IdbError> {
    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            let msg = "Window is null";
            return Err(IdbError::EnvError(msg.into()));
        }
    };

    let idb_factory = match window.indexed_db() {
        Ok(f_res) => match f_res {
            Some(f) => f,
            None => {
                let msg = "window.indexed_db is null";
                return Err(IdbError::EnvError(msg.into()));
            }
        },
        Err(e) => {
            let msg = e
                .as_string()
                .expect("cannot get IdbFactory from environment");
            return Err(IdbError::IdbFactoryError(msg));
        }
    };

    let _idb_open_request = idb_factory.open(name);

    Ok(())
}
