//! `ha observer` (Go: `cmd/observer*.go`).

mod info;
mod stats;
mod update;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("observer")
        .short("Get information, update or configure the Home Assistant observer")
        .long(
            "
The observer command allows you to manage the internal Home Assistant observer by
exposing commands to view, monitor, configure and control it.",
        )
        .example(
            "
  ha observer info
  ha observer update",
        )
        .subcommand(info::command())
        .subcommand(stats::command())
        .subcommand(update::command())
}
