use std::collections::HashMap;
use std::time::Duration;

use serde_derive::{Deserialize, Serialize};
use tokio::stream::StreamExt;

use rqueue::Codec;

#[derive(Serialize, Deserialize)]
struct DeliverSMMandatory {
    short_message: String,
}

#[derive(Serialize, Deserialize)]
struct Pdu {
    command_id: i64,
    command_status: i32,
    mandatory: DeliverSMMandatory,
    optional: HashMap<i32, String>,
    sequence_number: i32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    message_center_id: i32,
    pdus: std::vec::Vec<Pdu>,
    mtype: String,
    uid: String,
}

fn message() -> Message {
    Message {
        message_center_id: 3,
        pdus: vec![Pdu {
            command_id: 2147483648 as i64,
            command_status: 3,
            mandatory: DeliverSMMandatory {
                short_message: String::new(),
            },
            optional: HashMap::new(),
            sequence_number: 100003,
        }],
        mtype: "deliver_sm".to_owned(),
        uid: String::from(ulid::Ulid::new()),
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("running pop stream example");

    let config_json = rqueue::QueueConfig {
        codec: Codec::Json,
        queue_names: vec![String::from("rqueue-rs-stream-example")],
        ..Default::default()
    };

    let config_msgpack = rqueue::QueueConfig {
        codec: Codec::MsgPack,
        ..config_json.clone()
    };

    let queue_listener1 = rqueue::AsyncPersistentQueue::new(&config_msgpack)
        .await
        .unwrap();

    let queue_listener2 = rqueue::AsyncPersistentQueue::new(&config_msgpack)
        .await
        .unwrap();

    let stream1 = queue_listener1.into_stream::<serde_json::Value>();
    let stream2 = queue_listener2.into_stream::<serde_json::Value>();

    let mut push_queue = rqueue::AsyncPersistentQueue::new(&config_msgpack)
        .await
        .unwrap();

    tokio::pin!(stream1);
    tokio::pin!(stream2);

    let mut merged = stream1.merge(stream2);

    tokio::join!(
        async {
            loop {
                let _ = push_queue.push(&message(), 0).await;
                tokio::time::delay_for(Duration::from_secs(1)).await;
            }
        },
        async {
            while let Some(msg) = merged.next().await {
                log::info!("got {:?}", msg);
            }
        }
    );
}
