use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information about the Home Assistant Security backend")
        .long(
            "
This command provides general information about the Home Assistant Security backend.
",
        )
        .example(
            "
  ha security info
",
        )
        .run(Call::get("security", "info").run())
}
