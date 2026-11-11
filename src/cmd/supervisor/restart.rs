use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("restart")
        .aliases(&["reboot"])
        .short("Restart the Home Assistant Supervisor")
        .long(
            "
Restart the Supervisor internal, this can solve healthy issues.",
        )
        .example(
            "
  ha supervisor restart",
        )
        // Unlike the other restart commands, Go does not use the spinner here.
        .run(
            Call::post("supervisor", "restart")
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .run(),
        )
}
