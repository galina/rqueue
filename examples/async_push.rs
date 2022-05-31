use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("running async push example");

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

    let mut queue = rqueue::AsyncPersistentQueue::new(&config)
        .await
        .unwrap();

    let result = queue.push(&message, 0).await;
    log::info!("pushed into {}", result.unwrap());

    let popped = queue
        .pop::<serde_json::Value>(Duration::from_secs(1))
        .await
        .unwrap();
    log::info!("popped {:?}", popped);

    let _ = queue.push(&message, 0).await;
    let (name, length): (String, u32) = queue.length(None).await.unwrap();
    log::info!("{} queue length after push {}", name, length);

    let _ = queue.clean().await.unwrap();

    let (cn, cl): (String, u32) = queue.length(None).await.unwrap();
    log::info!("{} queue length after clean is {}", cn, cl);
}
