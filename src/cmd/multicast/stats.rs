use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("stats")
        .aliases(&["status", "stat"])
        .short("Provides system usage stats of the Home Assistant Multicast server")
        .long(
            "
Provides insight into the system usage stats of the Home Assistant Multicast server.
It shows you how much CPU, memory, disk & network resources it uses.
",
        )
        .example(
            "
  ha multicast stats
",
        )
        .run(Call::get("multicast", "stats").run())
}
