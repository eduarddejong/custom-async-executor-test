use std::{future::Future, pin, sync::Arc, task::Poll};

use crossbeam_channel::Sender;
use futures::task::{self, ArcWake};

struct Wake {
    sender: Sender<()>,
}

impl ArcWake for Wake {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self
            .sender
            .send(())
            .expect("Send through channel failed");
    }
}

pub fn block_on<T>(mut future: (impl Future<Output = T> + Sync + Send)) -> T {
    let mut future = pin::pin!(future);
    let (sender, receiver) = crossbeam_channel::bounded(1);
    let wake = Arc::new(Wake { sender });
    let waker = task::waker_ref(&wake);
    let context = &mut task::Context::from_waker(&waker);
    loop {
        let poll = future.as_mut().poll(context);
        if let Poll::Ready(result) = poll {
            return result;
        }
        receiver.recv().expect("Receive from channel failed");
    }
}
