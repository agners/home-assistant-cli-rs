//! `ha os boards raspberrypi` (Go: `cmd/os_boards_raspberrypi*.go`).

mod firmware;
mod firmware_update;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("raspberrypi")
        .aliases(&["rpi"])
        .short("See or change settings of the current Raspberry Pi board")
        .long(
            "
This command allows you to see or change settings of the Raspberry Pi board that
Home Assistant is running on.",
        )
        .example(
            "
  ha os boards raspberrypi firmware",
        )
        .subcommand(firmware::command())
}
