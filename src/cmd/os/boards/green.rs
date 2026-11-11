use super::green_options;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("green")
        .aliases(&["grn"])
        .short("See or change settings of the current Green board")
        .long(
            "
This command allows you to see or change settings of the Green board that Home
Assistant is running on.",
        )
        .example(
            "
  ha os boards green",
        )
        .run(Call::get("os", "boards/green").run())
        .subcommand(green_options::command())
}
