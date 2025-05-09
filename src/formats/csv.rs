use std::fs::{File, OpenOptions};

use csv::{Reader, Writer, WriterBuilder};

use crate::ARGS;

use super::{EventSink, LoginEvent};

pub struct CsvEventSink {
    writer: Writer<File>,
    entries: Vec<LoginEvent>,
}

impl CsvEventSink {
    pub fn new() -> Self {
        let file_name = format!("out/{}.csv", &ARGS.file_name);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_name)
            .unwrap();
        // if the reader is opened in append mode it will not read any entries
        let mut reader = Reader::from_path(&file_name).unwrap();
        let entries: Vec<LoginEvent> = reader.deserialize().filter_map(|res| res.ok()).collect();
        let writer = if entries.is_empty() {
            Writer::from_writer(file)
        } else {
            WriterBuilder::new().has_headers(false).from_writer(file)
        };
        Self { writer, entries }
    }
}

impl EventSink for CsvEventSink {
    fn write(&mut self, event: &super::LoginEvent) {
        self.entries.push(event.clone());
        self.writer.serialize(event).unwrap();
        self.writer.flush().unwrap();
    }

    fn name(&self) -> &'static str {
        "csv"
    }
}
