//! `ha supervisor` (Go: `cmd/supervisor*.go`).

mod available_updates;
mod info;
mod logs;
mod options;
mod reload;
mod repair;
mod restart;
mod stats;
mod update;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("supervisor")
        .aliases(&["super", "su"])
        .short("Monitor, control and configure the Home Assistant Supervisor")
        .long(
            "
The Home Assistant Supervisor is the heart of the Home Assistant system.
It manages your Home Assistant Core, Operating System, and all the apps.
It even manages itself! This series of command give you control over the
Home Assistant Supervisor.",
        )
        .example(
            "
  ha supervisor reload
  ha supervisor update
  ha supervisor logs",
        )
        .subcommand(available_updates::command())
        .subcommand(info::command())
        .subcommand(logs::command())
        .subcommand(options::command())
        .subcommand(reload::command())
        .subcommand(repair::command())
        .subcommand(restart::command())
        .subcommand(stats::command())
        .subcommand(update::command())
}
