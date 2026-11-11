use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("stats")
        .aliases(&["status", "stat", "st"])
        .short("Provides system usage stats of Home Assistant Audio")
        .long(
            "
Provides insight into the system usage stats of Home Assistant Audio.
It shows you how much CPU, memory, disk & network resources it uses.",
        )
        .example(
            "
  ha audio stats",
        )
        .run(Call::get("audio", "stats").run())
}
