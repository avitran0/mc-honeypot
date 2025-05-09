use rusqlite::Connection;

use crate::ARGS;

use super::EventSink;

pub struct SqliteEventSink {
    connection: Connection,
}

impl SqliteEventSink {
    pub fn new() -> Self {
        let connection = Connection::open(format!("{}.sqlite", &ARGS.file_name)).unwrap();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS login_events (
                ip TEXT,
                version INTEGER,
                mc_version TEXT,
                hostname TEXT,
                player_name TEXT,
                player_uuid TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
                (),
            )
            .unwrap();
        Self { connection }
    }
}

impl EventSink for SqliteEventSink {
    fn write(&mut self, event: &super::LoginEvent) {
        self.connection
            .execute(
                "INSERT INTO login_events
            (ip, version, mc_version, hostname, player_name, player_uuid)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                rusqlite::params![
                    event.ip.to_string(),
                    event.version,
                    event.mc_version,
                    event.hostname,
                    event.player_name,
                    event.player_uuid.to_string()
                ],
            )
            .unwrap();
    }

    fn name(&self) -> &'static str {
        "sqlite"
    }
}
