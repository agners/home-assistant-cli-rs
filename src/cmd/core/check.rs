use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("check")
        .aliases(&["validate", "chk", "ch"])
        .short("Validates your Home Assistant Core configuration")
        .long(
            "
This commands allows you to check/validate your, currently on disk stored,
Home Assistant Core configuration. This is helpful when you've made changes and
want to make sure the configuration is right, before restarting
Home Assistant Core.",
        )
        .example(
            "
  ha core check",
        )
        .run(
            Call::post("core", "check")
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .spinner()
                .run(),
        )
}
