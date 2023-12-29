use futures::task::{self, ArcWake};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    task::Poll,
};

struct SimpleExecutorArcWake<'a, T> {
    future: Mutex<Pin<&'a mut (dyn Future<Output = T> + Sync + Send)>>,
    sender: Mutex<Option<SyncSender<()>>>,
}

impl<'a, T> SimpleExecutorArcWake<'a, T> {
    pub fn execute(self: &Arc<Self>) -> T {
        let (sender, receiver) = mpsc::sync_channel(1);
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
        unreachable!("Receiver is closed, poll should be ready");
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

    pub fn block_on<T>(&mut self, future: Pin<&mut (impl Future<Output = T> + Sync + Send)>) -> T {
        Arc::new(SimpleExecutorArcWake {
            future: Mutex::new(future),
            sender: Mutex::new(None),
        })
        .execute()
    }
}
