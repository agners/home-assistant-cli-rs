use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information about the running Supervisor")
        .long(
            "
This command provides you a ton of information about everything the
Supervisor currently knows.",
        )
        .example(
            "
  ha supervisor info",
        )
        .run(Call::get("supervisor", "info").run())
}
