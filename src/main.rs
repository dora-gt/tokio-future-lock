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
    let updator_1 = MyDataUpdator::new(lock.clone());
    let updator_2 = MyDataUpdator::new(lock.clone());

    let mut runtime = RuntimeBuilder::new()
        .panic_handler(|err| std::panic::resume_unwind(err))
        .build()
        .unwrap();

    let fut_1 = updator_1.and_then(|mut guard|{
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
    let fut_2 = updator_2.and_then(|mut guard|{
        *guard *= 2;
        debug!("the value is now: {}", *guard);
        drop(guard);
        Ok(())
    });

    runtime.spawn(fut_1);
    runtime.block_on_all(fut_2);
}

struct MyDataUpdator {
    data: Lock<u8>,
}

impl MyDataUpdator {
    fn new(lock: Lock<u8>) -> Self {
        MyDataUpdator {
            data: lock,
        }
    }
}

impl Future for MyDataUpdator {
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
