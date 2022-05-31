use std::time::Duration;

use rqueue::Codec;

fn main() {
    env_logger::init();

    log::info!("running push/pop example");

    let pop_config = rqueue::QueueConfig {
        queue_names: vec![
            String::from("rqueue-rs-example-1"),
            String::from("rqueue-rs-example-2"),
        ],
        codec: Codec::MsgPack,
        ..Default::default()
    };

    let push_config = rqueue::QueueConfig {
        queue_names: vec![String::from("rqueue-rs-example-2")],
        codec: Codec::MsgPack,
        ..Default::default()
    };

    let mut push_queue = rqueue::PersistentQueue::new(&push_config).unwrap();
    let mut pop_queue = rqueue::PersistentQueue::new(&pop_config).unwrap();

    let handle = std::thread::spawn(move || loop {
        let _ = push_queue.push("data", 0);
        std::thread::sleep(Duration::from_secs(1));
    });

    let _ = std::thread::spawn(move || loop {
        let result = pop_queue.pop::<serde_json::Value>(Duration::from_secs(3));

        log::debug!("popped {:?}", result);
    });

    handle.join().unwrap();
}
