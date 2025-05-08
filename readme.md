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

## Todo

- [ ] file output (csv, json)
- [ ] database output (sqlite, others)
- [ ] configurable server appearance (motd, player count)
- [ ] configurable port
