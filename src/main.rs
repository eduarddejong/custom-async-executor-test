use custom_async_executor::SimpleExecutor;
use futures_time::time::Duration;

mod custom_async_executor;

fn main() {
    let async_main_future = async_main();
    futures::pin_mut!(async_main_future);
    SimpleExecutor::new().block_on(async_main_future);
}

async fn async_main() {
    println!("Hello, ");
    for i in (1..=5).rev() {
        println!("{i}...");
        futures_time::task::sleep(Duration::from_millis(1000)).await;
    }
    println!("world!");
}
