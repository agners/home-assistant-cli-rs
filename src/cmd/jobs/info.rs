use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information about the Home Assistant Job Manager")
        .long(
            "
This command provides general information about the Home Assistant Job backend.
",
        )
        .example(
            "
  ha jobs info
",
        )
        .run(Call::get("jobs", "info").run())
}
