use super::yellow_options;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("yellow")
        .aliases(&["yell"])
        .short("See or change settings of the current Yellow board")
        .long(
            "
This command allows you to see or change settings of the Yellow board that Home
Assistant is running on.",
        )
        .example(
            "
  ha os boards yellow",
        )
        .run(Call::get("os", "boards/yellow").run())
        .subcommand(yellow_options::command())
}
