//! `ha jobs` — Job Manager (Go: `cmd/jobs*.go`).

mod info;
mod options;
mod reset;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("jobs")
        .aliases(&["job", "tasks", "task"])
        .short("Get information and manage running jobs")
        .long(
            "
The jobs command allows you to manage the internal Home Assistant Job Manager and
exposing commands to view, configure and control it.",
        )
        .example(
            "
  ha jobs info
  ha jobs options",
        )
        .subcommand(info::command())
        .subcommand(options::command())
        .subcommand(reset::command())
}
