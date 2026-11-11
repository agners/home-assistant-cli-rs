use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows information about the internal Home Assistant CLI backend")
        .long(
            "
Shows information about the internally running Home Assistant CLI backend
",
        )
        .example(
            "
  ha cli info
",
        )
        .run(Call::get("cli", "info").run())
}
