//! `ha dns` — DNS plug-in management (Go: `cmd/dns*.go`).
//!
//! This group is the reference implementation for porting: one file per sub
//! command, each exposing `pub fn command() -> Command`.

mod info;
mod logs;
mod options;
mod reset;
mod restart;
mod stats;
mod update;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("dns")
        .short("Get information, update or configure the Home Assistant DNS server")
        .long(
            "
The dns command allows you to manage the internal Home Assistant DNS server by
exposing commands to view, monitor, configure and control it.",
        )
        .example(
            "
  ha dns logs
  ha dns info
  ha dns update",
        )
        .subcommand(info::command())
        .subcommand(logs::command())
        .subcommand(options::command())
        .subcommand(reset::command())
        .subcommand(restart::command())
        .subcommand(stats::command())
        .subcommand(update::command())
}
