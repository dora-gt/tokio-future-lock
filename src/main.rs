use tokio::runtime::Builder as RuntimeBuilder;
use futures::{future::join_all, Future, IntoFuture};
use futures::future::ok;
use log::{debug, Level};
use tokio::sync::lock::{Lock, LockGuard};
use tokio::prelude::Async;
use log::Level::Debug;
use std::sync::Arc;
use std::cell::RefCell;
use std::panic::resume_unwind;
use tokio::timer::Delay;
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    let lock = Lock::new(0);
    let updater_1 = MyDataUpdater::new(lock.clone());
    let updater_2 = MyDataUpdater::new(lock.clone());

    let mut runtime = RuntimeBuilder::new()
        .panic_handler(|err| std::panic::resume_unwind(err))
        .build()
        .unwrap();

    let fut_1 = updater_1.and_then(|mut guard|{
        Delay::new(Instant::now() + Duration::from_millis(1000))
            .map_err(|err| ())
            .and_then(move|_|{
                // guard could be moved into a closure
                *guard += 1;
                debug!("the value is now: {}", *guard);
                drop(guard);
                Ok(())
        })
    });
    let fut_2 = updater_2.and_then(|mut guard|{
        *guard *= 2;
        debug!("the value is now: {}", *guard);
        drop(guard);
        Ok(())
    });

    runtime.spawn(fut_1);
    runtime.block_on_all(fut_2);
}

struct MyDataUpdater {
    data: Lock<u8>,
}

impl MyDataUpdater {
    fn new(lock: Lock<u8>) -> Self {
        MyDataUpdater {
            data: lock,
        }
    }
}

impl Future for MyDataUpdater {
    type Item = LockGuard<u8>;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.data.poll_lock() {
            Async::Ready(mut guard) => {
                debug!("guarded: {:?}", guard);
                Ok(Async::Ready(guard))
            },
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}
