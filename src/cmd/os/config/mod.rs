//! `ha os config` (Go: `cmd/os_config*.go`).

mod swap;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("config")
        .aliases(&["conf", "cfg"])
        .short("Show or change Home Assistant OS settings")
        .long(
            "
This command allows you to show or change settings of Home Assistant OS.",
        )
        .example(
            "
  ha os config swap",
        )
        .subcommand(swap::command())
}
