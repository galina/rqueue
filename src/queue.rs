use bytes::BytesMut;
use std::time::Duration;
use std::vec::Vec;

use crate::codec::*;
use crate::config::*;
use crate::error::*;

#[derive(Debug)]
pub struct QueueEntry<T> {
    pub queue_name: String,
    pub data: T,
}

pub type QueueResult<T> = Result<T, QueueError>;

pub struct PersistentQueue {
    conn: redis::Connection,
    queue_names: Vec<String>,
    pop_rule: String,
    codec: Codec,
}

impl PersistentQueue {
    pub fn new(config: &QueueConfig) -> QueueResult<Self> {
        let client = redis::Client::open(&*config.redis_url)?;
        let conn = client.get_connection()?;
        let queue_names = config.queue_names.clone();
        let pop_rule = config.pop_rule.into_command();

        Ok(Self {
            conn,
            queue_names,
            pop_rule,
            codec: config.codec.clone(),
        })
    }

    pub fn push<T: ?Sized>(&mut self, data: &T, priority: i32) -> QueueResult<String>
    where
        T: serde::Serialize,
    {
        let encoded = self.codec.serialize(&data)?;
        let queue_name = self.queue_names.first().unwrap();

        let result: redis::RedisResult<()> = redis::cmd("ZADD")
            .arg(queue_name)
            .arg(priority)
            .arg(encoded)
            .query(&mut self.conn);

        match result {
            Ok(()) => Ok(String::from(queue_name)),
            Err(err) => Err(QueueError::from(err)),
        }
    }

    pub fn try_pop<T>(&mut self, timeout: Duration) -> QueueResult<Option<QueueEntry<T>>>
    where
        for<'a> T: serde::Deserialize<'a>,
    {
        let mut cmd = redis::cmd(&self.pop_rule);

        self.queue_names
            .iter()
            .fold(&mut cmd, |c, name| c.arg(name))
            .arg(timeout.as_secs())
            .query::<Option<(String, Vec<u8>, String)>>(&mut self.conn)
            .map_err(QueueError::from)
            .and_then(|result| match result {
                Some((queue_name, data, _)) => {
                    let decoded = self
                        .codec
                        .deserialize::<T>(&BytesMut::from(data.as_slice()));
                    match decoded {
                        Ok(data) => Ok(Some(QueueEntry { queue_name, data })),
                        Err(err) => Err(QueueError::from(err)),
                    }
                }
                None => Ok(None),
            })
    }

    pub fn pop<T>(&mut self, timeout: Duration) -> QueueResult<QueueEntry<T>>
    where
        for<'a> T: serde::Deserialize<'a>,
    {
        loop {
            let result = self.try_pop::<T>(timeout);

            match result {
                Ok(Some(result)) => return Ok(result),
                Ok(None) => continue,
                Err(err) => return Err(err),
            }
        }
    }
}
