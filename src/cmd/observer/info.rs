use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows information about the internal Home Assistant observer")
        .long(
            "
Shows information about the internally running Home Assistant observer
",
        )
        .example(
            "
  ha observer info
",
        )
        .run(Call::get("observer", "info").run())
}
