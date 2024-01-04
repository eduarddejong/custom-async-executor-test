use std::{
    future::Future,
    pin::{self, Pin},
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use crossbeam_channel::{Receiver, Sender};
use futures::task::{self, ArcWake};

enum Wake {
    Main(MainWake),
    Spawned(SpawnedWake),
}

struct MainWake {
    sender: Sender<Arc<Wake>>,
}

struct SpawnedWake {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    poll: Mutex<Poll<()>>,
    join_handle_waker: Mutex<Option<Waker>>,
    sender: Sender<Arc<Wake>>,
}

impl Wake {
    fn spawned_ref(&self) -> &SpawnedWake {
        match self {
            Self::Spawned(spawned_wake) => spawned_wake,
            _ => {
                panic!("Is not spawned");
            }
        }
    }

    fn sender(&self) -> &Sender<Arc<Wake>> {
        match self {
            Self::Main(main_wake) => &main_wake.sender,
            Self::Spawned(spawned_wake) => &spawned_wake.sender,
        }
    }
}

impl ArcWake for Wake {
    fn wake(self: Arc<Self>) {
        self.sender()
            .clone()
            .send(self)
            .expect("Send through channel failed");
    }

    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self
            .sender()
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
        let spawned_wake = self.wake.spawned_ref();

        // Do initial poll on future, so that code before the first await/poll inside it
        // is executed directly.
        let waker = task::waker_ref(&self.wake);
        let context = &mut Context::from_waker(&waker);
        *spawned_wake.poll.lock().unwrap() =
            spawned_wake.future.lock().unwrap().as_mut().poll(context);

        spawned_wake
            .sender
            .send(Arc::clone(&self.wake))
            .expect("Send through channel failed");
    }
}

impl<T> Future for SpawnHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let spawned_wake = self.wake.spawned_ref();

        if spawned_wake.poll.lock().unwrap().is_ready() {
            return Poll::Ready(
                self.result_receiver
                    .recv()
                    .expect("Receive through result channel failed"),
            );
        }

        *spawned_wake.join_handle_waker.lock().unwrap() = Some(cx.waker().clone());

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
        let mut wake = Arc::new(Wake::Main(MainWake {
            sender: self.sender.clone(),
        }));

        loop {
            match wake.as_ref() {
                // Main future poll
                Wake::Main(_) => {
                    let waker = task::waker_ref(&wake);
                    let context = &mut Context::from_waker(&waker);
                    let poll = future.as_mut().poll(context);
                    if let Poll::Ready(result) = poll {
                        return result;
                    }
                }
                // Spawned future poll
                Wake::Spawned(spawned_wake) => {
                    let waker = task::waker_ref(&wake);
                    let context = &mut Context::from_waker(&waker);
                    let mut poll = spawned_wake.poll.lock().unwrap();
                    if poll.is_pending() {
                        *poll = spawned_wake.future.lock().unwrap().as_mut().poll(context);
                        if poll.is_ready() {
                            if let Some(waker) =
                                spawned_wake.join_handle_waker.lock().unwrap().as_ref()
                            {
                                waker.wake_by_ref();
                            }
                        }
                    }
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
        let wake = Arc::new(Wake::Spawned(SpawnedWake {
            future: Mutex::new(Box::pin(async move {
                let result = future.await;
                result_sender
                    .send(result)
                    .expect("Send through result channel failed");
            })),
            poll: Mutex::new(Poll::Pending),
            join_handle_waker: Mutex::new(None),
            sender: self.sender.clone(),
        }));

        let spawn_handle = SpawnHandle {
            wake,
            result_receiver,
        };
        spawn_handle.init();
        spawn_handle
    }
}
