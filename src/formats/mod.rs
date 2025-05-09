use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod json;
pub mod sqlite;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginEvent {
    pub ip: SocketAddr,
    pub version: i32,
    pub mc_version: String,
    pub hostname: String,
    pub player_name: String,
    pub player_uuid: Uuid,
}

pub trait EventSink {
    fn write(&mut self, event: &LoginEvent);
    fn name(&self) -> &'static str;
}

pub struct MultiSink {
    sinks: Vec<Box<dyn EventSink + Send>>,
}

impl MultiSink {
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    pub fn add_sink<S: EventSink + Send + 'static>(&mut self, sink: S) {
        self.sinks.push(Box::new(sink));
    }

    pub fn write(&mut self, event: &LoginEvent) {
        for sink in &mut self.sinks {
            sink.write(event);
        }
    }

    pub fn sink_names(&self) -> String {
        let names = self.sinks.iter().map(|sink| sink.name());
        let mut sink_names = String::with_capacity(names.len() * 4);
        for (index, name) in names.enumerate() {
            if index != 0 {
                sink_names.push(',');
            }
            sink_names.push_str(name);
        }
        sink_names
    }
}
