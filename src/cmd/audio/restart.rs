use crate::cmd::helpers::{leaf, run_restart};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("restart")
        .aliases(&["reboot"])
        .short("Restarts the internal Home Assistant Audio")
        .long("Restart the internal Home Assistant Audio")
        .example(
            "
  ha audio restart",
        )
        .run(run_restart("audio"))
}
