use custom_async_executor::SimpleExecutor;
use futures_time::time::Duration;

mod custom_async_executor;

fn main() {
    let async_main_future = async_main();
    futures::pin_mut!(async_main_future);
    SimpleExecutor::new().execute(async_main_future);
}

async fn async_main() {
    println!("Hello, ");
    futures_time::task::sleep(Duration::from_millis(1000)).await;
    println!("world!");
}
