[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/krossbar-logger.svg
[crates-url]: https://crates.io/crates/krossbar-logger
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/krossbar-platform/krossbar-bus/blob/main/LICENSE
[actions-badge]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml

# krossbar-logger

Krossbar logging service

## Usage:

```bash
Usage: krossbar-logger [OPTIONS]

Options:
-l, --log-level <LOG_LEVEL>
        Logger self log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE [default: DEBUG]
    --log-location <LOG_LOCATION>
        Log file location [default: /var/log/krossbar/krossbar.log]
-n, --num-bytes-rotate <NUM_BYTES_ROTATE>
        Max log file size in bytes [default: 1000000]
-k, --keep-num-files <KEEP_NUM_FILES>
        How many rotated log files to keep [default: 10]
-h, --help
        Print help
-V, --version
        Print version
```

