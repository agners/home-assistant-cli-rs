use crate::cmd::helpers::{leaf, run_restart};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("restart")
        .aliases(&["reboot"])
        .short("Restarts the internal Home Assistant DNS server")
        .long("Restart the internal Home Assistant DNS server running")
        .example(
            "
  ha dns restart",
        )
        .run(run_restart("dns"))
}
