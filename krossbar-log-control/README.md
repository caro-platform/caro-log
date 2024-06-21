[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/krossbar-log-control.svg
[crates-url]: https://crates.io/crates/krossbar-log-control
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/krossbar-platform/krossbar-bus/blob/main/LICENSE
[actions-badge]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/krossbar-platform/krossbar-log/actions/workflows/ci.yml

# krossbar-log-control

Krossbar log control tool

The tool allows listing connected clients, and change their log level interactively.

Note: Log level for a particular service changes until restarted. Use logger internal mechanism to
persistently change log level.

## Usage

```sh
krossbar-log-control [OPTIONS](https://docs.rs/krossbar-log-control/latest/krossbar_log_control/) <SUBCOMMAND>

OPTIONS:
    -h, --help                     Print help information
    -l, --log-level <LOG_LEVEL>    Self log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE [default:
                                   DEBUG]
    -V, --version                  Print version information

SUBCOMMANDS:
    help             Print this message or the help of the given subcommand(s)
    list             List connected services
    set-log-level    Change service log level
```

List connected services:
```sh
USAGE:
    krossbar-log-control list

OPTIONS:
    -h, --help    Print help information
```

Change service log level:
```sh
USAGE:
    krossbar-log-control set-log-level --service-name <SERVICE_NAME> --level <LEVEL>

OPTIONS:
    -h, --help                           Print help information
    -l, --level <LEVEL>                  Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    -s, --service-name <SERVICE_NAME>    Log files location
```
