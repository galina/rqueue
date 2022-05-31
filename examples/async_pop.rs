use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("running async pop example");

    let config = rqueue::QueueConfig {
        queue_names: vec![String::from("rqueue-rs-example")],
        ..Default::default()
    };

    let mut queue = rqueue::AsyncPersistentQueue::new(&config)
        .await
        .unwrap();

    let result = queue.pop::<serde_json::Value>(Duration::from_secs(1)).await;

    log::info!("popped {:?}", result);
}
