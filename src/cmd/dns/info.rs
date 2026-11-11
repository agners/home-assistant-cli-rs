use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows information about the internal Home Assistant DNS server")
        .long(
            "
Shows information about the internally running Home Assistant DNS server
",
        )
        .example(
            "
  ha dns info
",
        )
        .run(Call::get("dns", "info").run())
}
