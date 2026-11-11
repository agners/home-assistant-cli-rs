use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reload")
        .aliases(&["refresh", "re"])
        .short("Reload the Home Assistant Audio updating information")
        .long(
            "
Reloading the Home Assistant Audio, triggers the to regather
all data and devices it currently has.",
        )
        .example(
            "
  ha audio reload",
        )
        .run(Call::post("audio", "reload").run())
}
