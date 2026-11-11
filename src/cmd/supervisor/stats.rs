use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("stats")
        .aliases(&["status", "stat", "st"])
        .short("Provides system usage stats of the Home Assistant Supervisor")
        .long(
            "
Provides insight into the system usage stats of the Home Assistant Supervisor.
It shows you how much CPU, memory, disk & network resources it uses.",
        )
        .example(
            "
  ha supervisor stats",
        )
        .run(Call::get("supervisor", "stats").run())
}
