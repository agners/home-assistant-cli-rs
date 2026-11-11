use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Shows information about the host docker backend")
        .long(
            "
Shows information about the local Docker backend on the host system
",
        )
        .example(
            "
  ha docker info
",
        )
        .run(Call::get("docker", "info").run())
}
