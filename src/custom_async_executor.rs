use std::{
    future::Future,
    pin::{self, Pin},
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use crossbeam_channel::{Receiver, Sender};
use futures::task::{self, ArcWake};

struct Wake {
    future: Option<Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    poll: Option<Mutex<Poll<()>>>,
    join_handle_waker: Option<Mutex<Option<Waker>>>,
    sender: Sender<Arc<Wake>>,
}

impl ArcWake for Wake {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self
            .sender
            .send(Arc::clone(arc_self))
            .expect("Send through channel failed");
    }
}

struct SpawnHandle<T> {
    wake: Arc<Wake>,
    result_receiver: Receiver<T>,
}

impl<T> SpawnHandle<T> {
    fn init(&self) {
        // Do initial poll on future, so that code before the first await/poll inside it
        // is executed directly.
        let waker = task::waker_ref(&self.wake);
        let context = &mut Context::from_waker(&waker);
        *self.wake.poll.as_ref().unwrap().lock().unwrap() = self
            .wake
            .future
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .as_mut()
            .poll(context);

        self.wake
            .sender
            .send(Arc::clone(&self.wake))
            .expect("Send through channel failed");
    }
}

impl<T> Future for SpawnHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.wake.poll.as_ref().unwrap().lock().unwrap().is_ready() {
            return Poll::Ready(
                self.result_receiver
                    .recv()
                    .expect("Receive through result channel failed"),
            );
        }

        *self
            .wake
            .join_handle_waker
            .as_ref()
            .unwrap()
            .lock()
            .unwrap() = Some(cx.waker().clone());

        Poll::Pending
    }
}

pub struct SimpleExecutor {
    sender: Sender<Arc<Wake>>,
    receiver: Receiver<Arc<Wake>>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn block_on<T>(&self, mut future: impl Future<Output = T>) -> T {
        let mut future = pin::pin!(future);
        let mut wake = Arc::new(Wake {
            future: None,
            poll: None,
            join_handle_waker: None,
            sender: self.sender.clone(),
        });

        loop {
            if let Some(future) = &wake.future {
                // Spawned future poll
                let waker = task::waker_ref(&wake);
                let context = &mut Context::from_waker(&waker);
                let mut poll = wake.poll.as_ref().unwrap().lock().unwrap();
                if poll.is_pending() {
                    *poll = future.lock().unwrap().as_mut().poll(context);
                    if poll.is_ready() {
                        if let Some(waker) = wake
                            .join_handle_waker
                            .as_ref()
                            .unwrap()
                            .lock()
                            .unwrap()
                            .as_ref()
                        {
                            waker.wake_by_ref();
                        }
                    }
                }
            } else {
                // Main future poll
                let waker = task::waker_ref(&wake);
                let context = &mut Context::from_waker(&waker);
                let poll = future.as_mut().poll(context);
                if let Poll::Ready(result) = poll {
                    return result;
                }
            }

            wake = self.receiver.recv().expect("Receive from channel failed");
        }
    }

    pub fn spawn<F: Future + Send + 'static>(&self, future: F) -> impl Future<Output = F::Output>
    where
        F::Output: Send,
    {
        let (result_sender, result_receiver) = crossbeam_channel::bounded(1);
        let wake = Arc::new(Wake {
            future: Some(Mutex::new(Box::pin(async move {
                let result = future.await;
                result_sender
                    .send(result)
                    .expect("Send through result channel failed");
            }))),
            poll: Some(Mutex::new(Poll::Pending)),
            join_handle_waker: Some(Mutex::new(None)),
            sender: self.sender.clone(),
        });

        let spawn_handle = SpawnHandle {
            wake,
            result_receiver,
        };
        spawn_handle.init();
        spawn_handle
    }
}
