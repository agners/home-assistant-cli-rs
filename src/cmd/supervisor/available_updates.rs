use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("available-updates")
        .aliases(&["available_updates", "updates"])
        .short("Provides information about available updates")
        .long(
            "
This command provides you information about available updates.",
        )
        .example(
            "
  ha supervisor available-updates",
        )
        .run(Call::get("supervisor", "available_updates").run())
}
