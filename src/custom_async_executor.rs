use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Poll,
};

use futures::task::{self, ArcWake};

struct SimpleExecutorArcWake<'a, T>(Mutex<Pin<&'a mut (dyn Future<Output = T> + Sync + Send)>>);

impl<'a, T> SimpleExecutorArcWake<'a, T> {
    pub fn poll(self: &Arc<Self>) {
        let mut poll = Poll::<T>::Pending;
        while poll.is_pending() {
            let waker = task::waker_ref(self);
            let context = &mut task::Context::from_waker(&waker);
            poll = self.0.lock().unwrap().as_mut().poll(context);
        }
    }
}

impl<T> ArcWake for SimpleExecutorArcWake<'_, T> {
    fn wake(self: Arc<Self>) {
        todo!("Somehow never being called.")
        // self.poll();
    }

    fn wake_by_ref(_arc_self: &Arc<Self>) {
        unimplemented!("Use wake() instead.");
    }
}

pub struct SimpleExecutor;

impl SimpleExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute<T>(&mut self, future: Pin<&mut (impl Future<Output = T> + Sync + Send)>) {
        let arc_wake_clone = Arc::new(SimpleExecutorArcWake(Mutex::new(future)));
        arc_wake_clone.poll();
    }
}
