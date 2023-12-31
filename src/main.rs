use std::{fmt::Display, time::Duration};

mod custom_async_executor;

mod custom_async_timer;

fn main() {
    custom_async_executor::block_on(async_main());
}

async fn async_main() {
    println!("Hello, ");
    let async_counter1 = async_counter((1..=5).rev());
    let async_counter2 = async_counter((6..=10).rev());
    futures::join!(async_counter1, async_counter2);
    println!("world!");
}

async fn async_counter<T: Display>(range: impl Iterator<Item = T>) {
    for i in range {
        println!("{i}...");
        custom_async_timer::sleep(Duration::from_millis(1000)).await;
    }
}
