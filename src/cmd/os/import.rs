use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("import")
        .aliases(&["im", "sync", "load"])
        .short("Import configurations from a USB stick")
        .long(
            "
This commands triggers an import action from a connected USB stick with
configuration to load for the Home Assistant Operating System.
",
        )
        .example(
            "
  ha os import
",
        )
        .run(Call::post("os", "config/sync").run())
}
