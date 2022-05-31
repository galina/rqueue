fn main() {
    env_logger::init();

    log::info!("running push example");

    let config = rqueue::QueueConfig {
        queue_names: vec![String::from("rqueue-rs-example")],
        ..Default::default()
    };

    let message = serde_json::json!({
        "message_center_id": 3,
        "pdus": [
            {
                "command_id": 2147483648 as u32,
                "command_status": 3,
                "mandatory": {
                    "short_message": ""
                },
                "optional": {},
                "sequence_number": 100003
            }
        ],
        "type": "deliver_sm",
        "uid": String::from(ulid::Ulid::new())
    })
    .to_owned();

    let mut queue = rqueue::PersistentQueue::new(&config).unwrap();

    let result = queue.push(&message, 0);

    log::info!("pushed into {}", result.unwrap());
}
