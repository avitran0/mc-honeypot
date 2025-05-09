use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write, read_to_string},
};

use crate::ARGS;

use super::{EventSink, LoginEvent};

pub struct JsonEventSink {
    file: File,
    entries: Vec<LoginEvent>,
}

impl JsonEventSink {
    pub fn new() -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(format!("{}.json", &ARGS.file_name))
            .unwrap();
        let entries = serde_json::from_str(&read_to_string(&file).unwrap()).unwrap_or(Vec::new());
        Self { file, entries }
    }
}

impl EventSink for JsonEventSink {
    fn write(&mut self, event: &super::LoginEvent) {
        self.entries.push(event.clone());
        self.file.seek(SeekFrom::Start(0)).unwrap();
        writeln!(
            &mut self.file,
            "{}",
            serde_json::to_string(&self.entries).unwrap()
        )
        .unwrap();
    }

    fn name(&self) -> &'static str {
        "json"
    }
}
