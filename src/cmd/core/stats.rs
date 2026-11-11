use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("stats")
        .aliases(&["status", "stat", "st"])
        .short("Provides system usage stats of Home Assistant Core")
        .long(
            "
Provides insight into the system usage stats of Home Assistant Core.
It shows you how much CPU, memory, disk & network resources it uses.",
        )
        .example(
            "
  ha core stats",
        )
        .run(Call::get("core", "stats").run())
}
