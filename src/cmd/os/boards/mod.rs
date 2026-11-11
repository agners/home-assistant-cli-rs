//! `ha os boards` (Go: `cmd/os_boards*.go`).

mod green;
mod green_options;
mod raspberrypi;
mod yellow;
mod yellow_options;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("boards")
        .short("See or change settings of the current board")
        .long(
            "
This command allows you to see or change settings of the board that Home
Assistant is running on.",
        )
        .example(
            "
  ha os boards yellow",
        )
        .subcommand(green::command())
        .subcommand(raspberrypi::command())
        .subcommand(yellow::command())
}
