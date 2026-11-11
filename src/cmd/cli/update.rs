use crate::cmd::helpers::{leaf, run_update, version_flag};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("update")
        .aliases(&["upgrade", "downgrade", "up", "down"])
        .short("Updates the internal Home Assistant CLI backend")
        .long(
            "
Using this command you can upgrade or downgrade the internal Home Assistant\x20
CLI backend, to the latest version or the version specified.
",
        )
        .example(
            "
  ha cli update --version 5
",
        )
        .flag(version_flag())
        .run(run_update("cli"))
}
