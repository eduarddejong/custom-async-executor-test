use std::{
    future::Future,
    pin::Pin,
    task::Poll,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::Receiver;

struct SimpleTimer {
    duration: Duration,
    join_handle: Option<JoinHandle<()>>,
    poll_receiver: Option<Receiver<Poll<()>>>,
}

impl Future for SimpleTimer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker().clone();
        let duration = self.duration;
        if self.join_handle.is_none() {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.poll_receiver = Some(receiver);
            self.join_handle = Some(thread::spawn(move || {
                thread::sleep(duration);
                sender.send(Poll::Pending).unwrap();
                waker.wake_by_ref();
                sender.send(Poll::Ready(())).unwrap();
            }));
        }
        self.poll_receiver.as_ref().unwrap().recv().unwrap()
    }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    SimpleTimer {
        duration,
        join_handle: None,
        poll_receiver: None,
    }
}