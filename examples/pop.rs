use std::time::Duration;

fn main() {
    env_logger::init();

    log::info!(r#"running pop example. try ZADD rqueue-rs-example 1 "\"data\"""#);

    let config = rqueue::QueueConfig {
        queue_names: vec![String::from("rqueue-rs-example")],
        ..Default::default()
    };

    let mut queue = rqueue::PersistentQueue::new(&config).unwrap();

    let result = queue.try_pop::<serde_json::Value>(Duration::from_secs(5));

    log::info!("got {:?}", result);
}
