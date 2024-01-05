use std::{fmt::Display, time::Duration};

pub use custom_async_executor::SimpleExecutor;

pub mod custom_async_executor;

pub mod minimalist_custom_async_executor;

pub mod custom_async_join;

pub mod custom_async_timer;

fn main() {
    println!("<<<Using custom async executor>>>");
    let executor = SimpleExecutor::new();
    executor.block_on(async_test1(&executor));
    println!();
    println!("<<<Using minimalist custom async executor>>>");
    minimalist_custom_async_executor::block_on(async_test2());
    println!();
}

async fn async_test1(executor: &SimpleExecutor) {
    println!("Hello, ");
    let async_counter1 = executor.spawn(async_counter1());
    let async_counter2 = executor.spawn(async_counter2());
    let async_counter3 = executor.spawn(async_counter3());
    let async_counter4 = executor.spawn(async_counter4());
    async_counter1.await;
    async_counter2.await;
    async_counter3.await;
    async_counter4.await;
    println!("world!");
}

async fn async_test2() {
    println!("Hello, ");
    let async_counter1 = async_counter1();
    let async_counter2 = async_counter2();
    let async_counter3 = async_counter3();
    let async_counter4 = async_counter4();
    custom_async_join::join4(
        async_counter1,
        async_counter2,
        async_counter3,
        async_counter4,
    )
    .await;
    println!("world!");
}

async fn async_counter1() {
    async_counter("A", (1..=9).rev(), Duration::from_millis(100)).await
}
async fn async_counter2() {
    async_counter("B", (1..=7).rev(), Duration::from_millis(300)).await
}
async fn async_counter3() {
    async_counter("C", (1..=5).rev(), Duration::from_millis(500)).await
}
async fn async_counter4() {
    async_counter("D", (1..=3).rev(), Duration::from_millis(900)).await
}

async fn async_counter<T: Display>(name: &str, range: impl Iterator<Item = T>, interval: Duration) {
    for i in range {
        println!("({name}) {i}...");
        custom_async_timer::sleep(interval).await;
    }
}
