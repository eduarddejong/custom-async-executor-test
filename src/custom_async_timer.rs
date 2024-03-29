use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::Receiver;

struct SimpleTimer {
    duration: Duration,
    join_handle: Option<JoinHandle<()>>,
    poll_receiver: Option<Receiver<Poll<()>>>,
    poll: Poll<()>,
}

impl Future for SimpleTimer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.join_handle.is_none() {
            let waker = cx.waker().clone();
            let duration = self.duration;
            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.poll_receiver = Some(receiver);
            self.join_handle = Some(thread::spawn(move || {
                thread::sleep(duration);
                sender.send(Poll::Ready(())).unwrap();
                waker.wake();
            }));
        }

        self.poll = self
            .poll_receiver
            .as_ref()
            .unwrap()
            .try_recv()
            .unwrap_or(self.poll);
        self.poll
    }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    SimpleTimer {
        duration,
        join_handle: None,
        poll_receiver: None,
        poll: Poll::Pending,
    }
}
