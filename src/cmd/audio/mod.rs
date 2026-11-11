//! `ha audio` (Go: `cmd/audio*.go`).

mod default;
mod info;
mod logs;
mod profile;
mod reload;
mod restart;
mod stats;
mod update;
mod volume;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("audio")
        .aliases(&["snd", "sound"])
        .short("Audio device handling.")
        .long(
            "
Control audio devices.
",
        )
        .example(
            "
  ha audio info
\t",
        )
        .subcommand(default::command())
        .subcommand(info::command())
        .subcommand(logs::command())
        .subcommand(profile::command())
        .subcommand(reload::command())
        .subcommand(restart::command())
        .subcommand(stats::command())
        .subcommand(update::command())
        .subcommand(volume::command())
}
