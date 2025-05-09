# Minecraft Honeypot

a decoy server that records unwanted connections or malicious activity.

## Purpose

this honeypot simulates a few mineraft packets (ping and first login).
from this it can collect the ip, client version and player info of anyone who tries to connect.

## Installation

### Prerequisites

- Rust
- Git

```bash
git clone https://github.com/avitran0/mc-honeypot
cd mc-honeypot
cargo run
```

## Usage

`cargo run` to use with defaults (json format only, default minecraft server settings)

`cargo run -- --flags` to use with additional flags

- `--formats`: selects the output formats as a comma-separated list (json, csv, sqlite)
- `--file-name`: sets the output file name, without extension, because that is format-dependent
- `--port`: selects port the server listens on
- `--message`: sets the "message of the day" (has to be in quotes)
- `--max-players`: sets the maximum amount of players on the server
- `--online-players`: sets the amount of online players on the server

## Todo

- [x] file output (csv, json)
- [ ] database output (sqlite, others)
- [ ] customizable server icon
- [x] configurable server appearance (motd, player count)
- [x] configurable port
