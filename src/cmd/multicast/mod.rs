//! `ha multicast` (Go: `cmd/multicast*.go`).

mod info;
mod logs;
mod options;
mod restart;
mod stats;
mod update;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("multicast")
        .aliases(&["mcast", "mc"])
        .short("Get information, update or configure the Home Assistant Multicast")
        .long(
            "
The multicast command allows you to manage the internal Home Assistant Multicast
backend by exposing commands to view, monitor, configure and control it.",
        )
        .example(
            "
  ha multicast info
  ha multicast update
  ha multicast options --enabled=false",
        )
        .subcommand(info::command())
        .subcommand(logs::command())
        .subcommand(options::command())
        .subcommand(restart::command())
        .subcommand(stats::command())
        .subcommand(update::command())
}
