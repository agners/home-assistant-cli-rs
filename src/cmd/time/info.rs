use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows Home Assistant OS time configuration")
        .long(
            "
Shows the configured Home Assistant OS NTP servers.",
        )
        .example(
            "
  ha time info",
        )
        .run(Call::get("time", "info").run())
}
