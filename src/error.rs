#[derive(Debug)]
pub enum QueueError {
    RedisError(redis::RedisError),
    IOError(IOError),
}

#[derive(Debug)]
pub struct IOError {
    pub data: Option<Vec<u8>>,
    pub inner: std::io::Error,
}

impl IOError {
    pub fn new(data: Option<Vec<u8>>, err: std::io::Error) -> Self {
        Self {
            data: data,
            inner: err,
        }
    }
}

impl From<std::io::Error> for QueueError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(IOError::new(None, err))
    }
}

impl From<IOError> for QueueError {
    fn from(err: IOError) -> Self {
        Self::IOError(err)
    }
}

impl From<redis::RedisError> for QueueError {
    fn from(err: redis::RedisError) -> Self {
        Self::RedisError(err)
    }
}
