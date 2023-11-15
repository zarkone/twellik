use core::future::Future;
use core::pin::Pin;
use core::task;
use core::task::Poll;
use thiserror::Error;
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
    type Output = IndexedDB<'a>;
    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> task::Poll<Self::Output> {
        match self.open_db_request.ready_state() {
            IdbRequestReadyState::Pending => {
                let waker = ctx.waker();

                self.open_db_request.set_onsuccess(Some(|_e| {
                    waker.wake();
                }));

                Poll::Pending
            }
            IdbRequestReadyState::Done => Poll::Ready(self),
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

    let idb_open_request = idb_factory.open(name);

    Ok(())
}
