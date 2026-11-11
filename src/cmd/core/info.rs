use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information about Home Assistant Core")
        .long(
            "
This command provides information about the running Home Assistant Core instance
running on your Home Assistant system.",
        )
        .example(
            "
  ha core info",
        )
        .run(Call::get("core", "info").run())
}
