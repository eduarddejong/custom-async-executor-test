use std::{
    future::Future,
    pin::{self, Pin},
    sync::{Arc, Mutex},
    task::Poll,
};

use crossbeam_channel::Sender;
use futures::task::{self, ArcWake};

struct SimpleExecutorArcWake<'a, T> {
    future: Mutex<Pin<&'a mut (dyn Future<Output = T> + Sync + Send)>>,
    sender: Mutex<Option<Sender<()>>>,
}

impl<'a, T> SimpleExecutorArcWake<'a, T> {
    pub fn execute(self: &Arc<Self>) -> T {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        sender.send(()).unwrap();
        *self.sender.lock().unwrap() = Some(sender);
        while receiver.recv().is_ok() {
            let waker = task::waker_ref(self);
            let context = &mut task::Context::from_waker(&waker);
            let poll = self.future.lock().unwrap().as_mut().poll(context);
            if let Poll::Ready(result) = poll {
                *self.sender.lock().unwrap() = None;
                return result;
            }
        }
        unreachable!("Channel is closed, poll should be ready");
    }
}

impl<T> ArcWake for SimpleExecutorArcWake<'_, T> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        if let Some(sender) = arc_self.sender.lock().unwrap().as_ref() {
            sender.send(()).expect("Send through channel failed");
        }
    }
}

pub struct SimpleExecutor;

impl SimpleExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn block_on<T>(&mut self, mut future: (impl Future<Output = T> + Sync + Send)) -> T {
        let future = pin::pin!(future);
        Arc::new(SimpleExecutorArcWake {
            future: Mutex::new(future),
            sender: Mutex::new(None),
        })
        .execute()
    }
}
