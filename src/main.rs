use std::{fmt::Display, time::Duration};

use custom_async_executor::SimpleExecutor;

mod custom_async_executor;

mod custom_async_timer;

fn main() {
    let executor = SimpleExecutor::new();
    executor.block_on(async_main(&executor));
}

async fn async_main(executor: &SimpleExecutor) {
    println!("Hello, ");
    let async_counter1 = executor.spawn(async_counter((1..=5).rev()));
    let async_counter2 = executor.spawn(async_counter((6..=10).rev()));
    async_counter1.await;
    async_counter2.await;
    println!("world!");
}

async fn async_counter<T: Display>(range: impl Iterator<Item = T>) {
    for i in range {
        println!("{i}...");
        custom_async_timer::sleep(Duration::from_millis(500)).await;
    }
}
