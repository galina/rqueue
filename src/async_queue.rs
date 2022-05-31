use bytes::BytesMut;
use futures_core::stream::Stream;
use std::time::Duration;

use crate::codec::*;
use crate::config::*;
use crate::error::*;
use crate::queue::QueueEntry;

pub type AsyncQueueResult<T> = Result<T, QueueError>;

#[derive(Clone)]
pub struct AsyncPersistentQueue {
    conn: redis::aio::ConnectionManager,
    queue_names: Vec<String>,
    pop_rule: String,
    codec: Codec,
}

impl AsyncPersistentQueue {
    pub async fn new(config: &QueueConfig) -> AsyncQueueResult<Self> {
        let queue_names = config.queue_names.clone();
        let pop_rule = config.pop_rule.into_command();
        let client = redis::Client::open(&*config.redis_url)?;
        let conn = client.get_tokio_connection_manager().await?;

        Ok(Self {
            conn,
            queue_names,
            pop_rule,
            codec: config.codec.clone(),
        })
    }

    pub async fn push<T: ?Sized>(&mut self, data: &T, priority: i32) -> AsyncQueueResult<String>
    where
        T: serde::Serialize,
    {
        let encoded = self.codec.serialize(&data)?;
        let queue_name = self.queue_names.first().unwrap();

        redis::cmd("ZADD")
            .arg(queue_name)
            .arg(priority)
            .arg(encoded)
            .query_async::<redis::aio::ConnectionManager, i32>(&mut self.conn)
            .await
            .map(|_| String::from(queue_name))
            .map_err(QueueError::from)
    }

    pub async fn length(&mut self, queue_name: Option<String>) -> AsyncQueueResult<(String, u32)> {
        let name = queue_name.unwrap_or_else(|| self.queue_names.first().unwrap().to_owned());

        redis::cmd("ZCARD")
            .arg(&name)
            .query_async::<redis::aio::ConnectionManager, u32>(&mut self.conn)
            .await
            .map_err(QueueError::from)
            .map(|res| (name, res))
    }

    pub async fn clean(&mut self) -> AsyncQueueResult<()> {
        let mut cmd = redis::cmd("DEL");

        self.queue_names
            .iter()
            .fold(&mut cmd, |c, name| c.arg(name))
            .query_async::<redis::aio::ConnectionManager, ()>(&mut self.conn)
            .await
            .map_err(QueueError::from)
    }

    pub async fn pop<T>(&mut self, timeout: Duration) -> AsyncQueueResult<Option<QueueEntry<T>>>
    where
        for<'a> T: serde::Deserialize<'a>,
    {
        let mut cmd = redis::cmd(&self.pop_rule);

        self.queue_names
            .iter()
            .fold(&mut cmd, |c, name| c.arg(name))
            .arg(timeout.as_secs())
            .query_async::<redis::aio::ConnectionManager, Option<(String, Vec<u8>, String)>>(
                &mut self.conn,
            )
            .await
            .map_err(QueueError::from)
            .and_then(|result| match result {
                Some((queue_name, data, _n)) => {
                    let decoded = self
                        .codec
                        .deserialize::<T>(&BytesMut::from(data.as_slice()));
                    match decoded {
                        Ok(data) => Ok(Some(QueueEntry { queue_name, data })),
                        Err(err @ std::io::Error { .. }) => {
                            Err(QueueError::IOError(IOError::new(Some(data), err)))
                        }
                    }
                }
                None => Ok(None),
            })
    }

    pub fn into_stream<T>(mut self) -> impl Stream<Item = AsyncQueueResult<QueueEntry<T>>>
    where
        for<'a> T: serde::Deserialize<'a> + Unpin,
    {
        async_stream::stream! {
            loop {
                let result = self.pop::<T>(Duration::from_secs(3)).await;

                match result {
                    Ok(Some(result)) => yield Ok(result),
                    Ok(None) => continue,
                    // При проблемах с подключением к редису попытки чтения выполняются с некоторой
                    // фиксированной задержкой чтобы не создавать лишнюю нагрузку на CPU при
                    // попытках ConnectionManager переустановить подключение к редису
                    Err(QueueError::RedisError(redis_err)) if redis_err.kind() == redis::ErrorKind::IoError => tokio::time::delay_for(Duration::from_millis(100)).await,
                    Err(err) => yield Err(err)
                }
            }
        }
    }
}
