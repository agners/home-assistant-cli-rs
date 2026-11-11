//! `ha security` — Security backend (Go: `cmd/security*.go`).

mod info;
mod integrity;
mod options;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("security")
        .aliases(&["secure", "sec"])
        .short("Get information and manage security functionality")
        .long(
            "
The security command allows you to manage the internal Home Assistant Security backend and
exposing commands to view, configure and control it.",
        )
        .example(
            "
  ha security info
  ha security options",
        )
        .subcommand(info::command())
        .subcommand(integrity::command())
        .subcommand(options::command())
}
