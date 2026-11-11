use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("stats")
        .aliases(&["status", "stat"])
        .short("Provides system usage stats of the Home Assistant DNS server")
        .long(
            "
Provides insight into the system usage stats of the Home Assistant DNS server.
It shows you how much CPU, memory, disk & network resources it uses.
",
        )
        .example(
            "
  ha dns stats
",
        )
        .run(Call::get("dns", "stats").run())
}
