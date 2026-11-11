use crate::cmd::helpers::{leaf, run_restart};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("restart")
        .aliases(&["reboot"])
        .short("Restarts the internal Home Assistant Multicast server")
        .long("Restart the internal Home Assistant Multicast server running")
        .example(
            "
  ha multicast restart",
        )
        .run(run_restart("multicast"))
}
