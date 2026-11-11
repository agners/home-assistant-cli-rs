use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows information about the internal Home Assistant Multicast server")
        .long(
            "
Shows information about the internally running Home Assistant Multicast server
",
        )
        .example(
            "
  ha multicast info
",
        )
        .run(Call::get("multicast", "info").run())
}
