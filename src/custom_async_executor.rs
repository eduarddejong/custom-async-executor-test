use std::{
    future::Future,
    pin::{self, Pin},
    sync::{Arc, Mutex},
    task::Poll,
};

use crossbeam_channel::{Receiver, Sender};
use futures::task::{self, ArcWake};

struct SimpleExecutorArcWake<'a, T> {
    future: Mutex<Pin<&'a mut (dyn Future<Output = T> + Sync + Send)>>,
    sender: Sender<()>,
    receiver: Receiver<()>,
}

impl<'a, T> SimpleExecutorArcWake<'a, T> {
    fn execute(self: &Arc<Self>) -> T {
        self.sender.send(()).unwrap();
        while self.receiver.recv().is_ok() {
            let waker = task::waker_ref(self);
            let context = &mut task::Context::from_waker(&waker);
            let poll = self.future.lock().unwrap().as_mut().poll(context);
            if let Poll::Ready(result) = poll {
                return result;
            }
        }
        unreachable!("Poll should be ready, result should be returned");
    }
}

impl<T> ArcWake for SimpleExecutorArcWake<'_, T> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self
            .sender
            .send(())
            .expect("Send through channel failed");
    }
}

pub struct SimpleExecutor;

impl SimpleExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn block_on<T>(&mut self, mut future: (impl Future<Output = T> + Sync + Send)) -> T {
        let future = pin::pin!(future);
        let (sender, receiver) = crossbeam_channel::bounded(1);
        Arc::new(SimpleExecutorArcWake {
            future: Mutex::new(future),
            sender,
            receiver,
        })
        .execute()
    }
}
