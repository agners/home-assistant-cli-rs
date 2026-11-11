use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("stats")
        .aliases(&["status", "stat"])
        .short("Provides system usage stats of the Home Assistant CLI backend")
        .long(
            "
Provides insight into the system usage stats of the Home Assistant CLI backend.
It shows you how much CPU, memory, disk & network resources it uses.
",
        )
        .example(
            "
  ha cli stats
",
        )
        .run(Call::get("cli", "stats").run())
}
