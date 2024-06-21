[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/krossbar-log-lib.svg
[crates-url]: https://crates.io/crates/krossbar-log-lib
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/krossbar-platform/krossbar-bus/blob/main/LICENSE
[actions-badge]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml

# krossbar-log-lib

Krossbar logging library

The library is used to connect to Krossbar logging service to send log messages
and receive log control commands.

See [Krossbar log control](https://crates.io/crates/krossbar-log-control) documentation
on how to control logging.

The library uses Unix stream connection to send logging messages, which means you need
running [Krossbar logger](https://crates.io/crates/krossbar-logger) to log mesage.
In case service can't connect to the logger, it logs to stdout.

Also, you can use [Logger](https://docs.rs/krossbar-log-lib/latest/krossbar_log_lib/logger/struct.Logger.html) manually to control whether log into stdout or send
message to the logger. Both option are independent.

In case you use Krossbar logger, you have to run logging loop using [Logger::run](https://docs.rs/krossbar-log-lib/latest/krossbar_log_lib/logger/struct.Logger.html#method.run).

## Examples
```rust
use std::time::Duration;

use log::*;

use krossbar_log_lib::init_logger;
use tokio::select;

#[tokio::main]
async fn main() {
    let logger = init_logger("com.examples.logging", LevelFilter::Trace, true).await;

    tokio::spawn(logger.run());

    loop {
        error!("Error message");
        warn!("Warning message");
        info!("Info message");
        debug!("Debug message");
        trace!("Trace message");

        select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {},
            _ = tokio::signal::ctrl_c() => { return; }
        }
    }
}
```
