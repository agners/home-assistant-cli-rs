//! `ha time` — Home Assistant OS time settings (Go: `cmd/time*.go`).

mod info;
mod options;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("time")
        .aliases(&["ntp", "timedate"])
        .short("Get information or configure Home Assistant OS time settings")
        .long(
            "
The time command allows you to view and configure the Home Assistant OS network
time synchronization servers.",
        )
        .example(
            "
  ha time info
  ha time options --servers pool.ntp.org --fallback-servers time.google.com",
        )
        .subcommand(info::command())
        .subcommand(options::command())
}
