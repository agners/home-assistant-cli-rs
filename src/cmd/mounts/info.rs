use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows information about mounts")
        .long(
            "
Shows information about the currently configured mounts in Supervisor
",
        )
        .example(
            "
  ha mounts info
",
        )
        .run(Call::get("mounts", "").run())
}
