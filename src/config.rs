use std::vec::Vec;

use crate::codec::*;

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PopRule {
    PopMin,
    PopMax,
}

impl Default for PopRule {
    fn default() -> Self {
        PopRule::PopMin
    }
}

impl PopRule {
    pub(crate) fn into_command(&self) -> String {
        match self {
            PopRule::PopMin => String::from("BZPOPMIN"),
            PopRule::PopMax => String::from("BZPOPMAX"),
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct QueueConfig {
    pub redis_url: String,
    pub queue_names: Vec<String>,
    #[serde(default)]
    pub pop_rule: PopRule,
    #[serde(default)]
    pub codec: Codec,
}

impl QueueConfig {
    pub fn with_queue(&mut self, queue_name: &str) {
        self.queue_names.insert(0, String::from(queue_name))
    }
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            redis_url: String::from("redis://127.0.0.1:6379/0"),
            queue_names: Vec::<String>::new(),
            pop_rule: PopRule::default(),
            codec: Codec::default(),
        }
    }
}
