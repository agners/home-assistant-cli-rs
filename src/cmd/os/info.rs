use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information about the running Home Assistant Operating System")
        .long(
            "
This command provides general information about the running Home Assistant Operating System.
",
        )
        .example(
            "
  ha os info
",
        )
        .run(Call::get("os", "info").run())
}
