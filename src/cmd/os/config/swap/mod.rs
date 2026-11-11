//! `ha os config swap` (Go: `cmd/os_config_swap*.go`).

mod info;
mod options;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("swap")
        .aliases(&["sw"])
        .short("Show or change Home Assistant OS swap settings")
        .long(
            "
This command allows you to show or change current swap configuration
of Home Assistant OS.",
        )
        .example(
            "
  ha os config swap info",
        )
        .subcommand(info::command())
        .subcommand(options::command())
}
