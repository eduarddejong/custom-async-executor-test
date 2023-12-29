use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
    task::Poll,
    thread::{self, JoinHandle},
    time::Duration,
};

struct SimpleTimer {
    duration: Duration,
    join_handle: Option<JoinHandle<()>>,
    poll_receiver: Mutex<Option<Receiver<Poll<()>>>>,
}

impl Future for SimpleTimer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker().clone();
        let duration = self.duration;
        if self.join_handle.is_none() {
            let (sender, receiver) = mpsc::sync_channel(1);
            *self.poll_receiver.lock().unwrap() = Some(receiver);
            self.join_handle = Some(thread::spawn(move || {
                thread::sleep(duration);
                sender.send(Poll::Pending).unwrap();
                waker.wake_by_ref();
                sender.send(Poll::Ready(())).unwrap();
            }));
        }
        self.poll_receiver
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .recv()
            .unwrap()
    }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    SimpleTimer {
        duration,
        join_handle: None,
        poll_receiver: Mutex::new(None),
    }
}
