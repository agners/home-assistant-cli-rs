use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("repair")
        .aliases(&["rep", "fix"])
        .short("Repair Docker issue automatically using the Supervisor")
        .long(
            "
There are cases where the Docker file system running on your Home Assistant
system, encounters issue or corruptions. Running this command,
the Home Assistant Supervisor will try to resolve these.
",
        )
        .example(
            "
  ha supervisor repair",
        )
        .run(
            Call::post("supervisor", "repair")
                .timeout(client::CONTAINER_DOWNLOAD_TIMEOUT)
                .spinner()
                .run(),
        )
}
