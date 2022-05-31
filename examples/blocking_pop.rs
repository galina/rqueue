use std::time::Duration;

fn main() {
    env_logger::init();

    log::info!("running blocking pop example");

    let config = rqueue::QueueConfig {
        queue_names: vec![String::from("rqueue-rs-example")],
        ..Default::default()
    };

    let mut queue = rqueue::PersistentQueue::new(&config).unwrap();

    log::info!(r#"awaiting for data. try ZADD rqueue-rs-example 1 "\"data\"""#);

    let result = queue.pop::<serde_json::Value>(Duration::from_secs(3));

    log::info!("got {:?}", result);
}
