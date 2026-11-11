use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information about Home Assistant Audio devices")
        .long(
            "
This command provides information about the running Home Assistant Audio instance
running on your Home Assistant system, including its devices.",
        )
        .example(
            "
  ha audio info",
        )
        .run(Call::get("audio", "info").run())
}
