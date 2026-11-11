//! `ha host` — host system control (Go: `cmd/host*.go`).
//!
//! The shared logs helpers of `cmd/host.go` (`addLogsFlags`,
//! `processLogsFlags`) live in `crate::cmd::logs`.

mod disks;
mod disks_usage;
mod info;
mod logs;
mod logs_boots;
mod logs_identifiers;
mod options;
mod reboot;
mod reload;
mod shutdown;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("host")
        .aliases(&["ho"])
        .short("Control the host/system that Home Assistant is running on")
        .long(
            "
The host command provides commandline tools to control the host (system) that
Home Assistant is running on. It allows you do thing like reboot or shutdown the
system, but also provides option to change the hostname of the system.",
        )
        .example(
            "
  ha host reboot
  ha host options --hostname \"homeassistant.local\"
",
        )
        .subcommand(disks::command())
        .subcommand(info::command())
        .subcommand(logs::command())
        .subcommand(options::command())
        .subcommand(reboot::command())
        .subcommand(reload::command())
        .subcommand(shutdown::command())
}
