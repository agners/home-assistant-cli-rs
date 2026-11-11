//! `ha cli` (Go: `cmd/cli*.go`).

mod info;
mod stats;
mod update;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("cli")
        .short("Get information, update or configure the Home Assistant cli backend")
        .long(
            "
The cli command allows you to manage the internal Home Assistant CLI backend by
exposing commands to view, monitor, configure and control it.",
        )
        .example(
            "
  ha cli info
  ha cli update",
        )
        .subcommand(info::command())
        .subcommand(stats::command())
        .subcommand(update::command())
}
