use std::{fmt::Display, time::Duration};

pub use custom_async_executor::SimpleExecutor;

pub mod custom_async_executor;

pub mod minimalist_custom_async_executor;

pub mod custom_async_join;

pub mod custom_async_timer;

fn main() {
    let executor = SimpleExecutor::new();
    executor.block_on(async_main(&executor));
}

async fn async_main(executor: &SimpleExecutor) {
    println!("Hello, ");
    let async_counter1 = executor.spawn(async_counter(
        "A",
        (1..=9).rev(),
        Duration::from_millis(100),
    ));
    let async_counter2 = executor.spawn(async_counter(
        "B",
        (1..=7).rev(),
        Duration::from_millis(300),
    ));
    let async_counter3 = executor.spawn(async_counter(
        "C",
        (1..=5).rev(),
        Duration::from_millis(500),
    ));
    let async_counter4 = executor.spawn(async_counter(
        "D",
        (1..=3).rev(),
        Duration::from_millis(900),
    ));
    async_counter1.await;
    async_counter2.await;
    async_counter3.await;
    async_counter4.await;
    println!("world!");
}

async fn async_counter<T: Display>(name: &str, range: impl Iterator<Item = T>, interval: Duration) {
    for i in range {
        println!("({name}) {i}...");
        custom_async_timer::sleep(interval).await;
    }
}
