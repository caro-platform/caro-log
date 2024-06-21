[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/krossbar-log-viewer.svg
[crates-url]: https://crates.io/crates/krossbar-log-viewer
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/krossbar-platform/krossbar-bus/blob/main/LICENSE
[actions-badge]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml

# krossbar-log-viewer

A tool to view Krossbar logs.

Although Krossbar logs are plain text files, the viewer sticks rotated
log files allowing to watch whole log sequence, and highlights log messages
section to simplify visual monitoring.

There're two modes: viewing ready logs; and interactive mode to see logs
as they appear. The interactive mode can be enables using **-f|--follow** CLI param.

## Usage
```bash
Usage: krossbar-log-viewer [OPTIONS]

Options:
-l, --log-level <LOG_LEVEL>        Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE [default: INFO]
    --log-location <LOG_LOCATION>  Log files location [default: /var/log/krossbar/krossbar.log]
-f, --follow                       Output appended data as the file grows
-h, --help                         Print help
-V, --version                      Print version
```
