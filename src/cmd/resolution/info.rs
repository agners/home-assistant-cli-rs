use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Show issues and suggestions")
        .long(
            "
This command provides general information about the issues, suggestion and the supported state of the system.",
        )
        .example(
            "
  ha resolution info",
        )
        .run(Call::get("resolution", "info").run())
}
