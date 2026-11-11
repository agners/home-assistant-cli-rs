use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("start")
        .aliases(&["run", "st"])
        .short("Manually start Home Assistant Core")
        .long(
            "
This command allows you to manually start the Home Assistant Core instance on
your system. This, of course, only applies when it has been stopped.",
        )
        .example(
            "
  ha core start",
        )
        .run(
            Call::post("core", "start")
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .spinner()
                .run(),
        )
}
