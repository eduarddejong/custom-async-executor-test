use custom_async_executor::SimpleExecutor;
use std::time::Duration;

mod custom_async_executor;

mod custom_async_timer;

fn main() {
    let async_main_future = async_main();
    futures::pin_mut!(async_main_future);
    SimpleExecutor::new().block_on(async_main_future);
}

async fn async_main() {
    println!("Hello, ");
    for i in (1..=5).rev() {
        println!("{i}...");
        custom_async_timer::sleep(Duration::from_millis(1000)).await;
    }
    println!("world!");
}
