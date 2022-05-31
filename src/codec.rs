use bytes::{Buf, BytesMut};
use std::io;

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    Json,
    MsgPack,
}

impl Default for Codec {
    fn default() -> Self {
        Codec::Json
    }
}

impl Codec {
    pub fn serialize<T>(&self, data: &T) -> Result<Vec<u8>, io::Error>
    where
        T: serde::Serialize,
    {
        match self {
            Codec::Json => {
                serde_json::to_vec(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            }
            Codec::MsgPack => rmp_serde::to_vec_named(data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn deserialize<T>(&self, src: &BytesMut) -> Result<T, io::Error>
    where
        for<'a> T: serde::Deserialize<'a>,
    {
        match self {
            Codec::Json => serde_json::from_reader(io::Cursor::new(src).reader())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Codec::MsgPack => rmp_serde::from_read(io::Cursor::new(src).reader())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}
