use std::{
    future::Future,
    pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crossbeam_channel::Sender;

fn create_waker(sender: &Sender<()>) -> Waker {
    unsafe {
        Waker::from_raw(RawWaker::new(
            sender as *const _ as *const _,
            create_raw_waker_vtable(),
        ))
    }
}

fn create_raw_waker_vtable() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        raw_waker_clone,
        raw_waker_wake,
        raw_waker_wake_by_ref,
        raw_waker_drop,
    )
}

unsafe fn raw_waker_clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, create_raw_waker_vtable())
}

unsafe fn raw_waker_wake(data: *const ()) {
    raw_waker_wake_by_ref(data);
}

unsafe fn raw_waker_wake_by_ref(data: *const ()) {
    let sender: &Sender<()> = &*(data as *const _);
    sender.send(()).expect("Sending through channel failed")
}

unsafe fn raw_waker_drop(_data: *const ()) {}

pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin::pin!(future);

    let (sender, receiver) = crossbeam_channel::bounded::<()>(1);

    let waker: Waker = create_waker(&sender);

    loop {
        let mut context = Context::from_waker(&waker);
        let poll = future.as_mut().poll(&mut context);
        if let Poll::Ready(result) = poll {
            return result;
        }
        receiver.recv().expect("Receiving from channel failed");
    }
}
