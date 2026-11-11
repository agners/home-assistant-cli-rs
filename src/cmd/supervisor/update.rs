use crate::cmd::helpers::{leaf, run_update, version_flag};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("update")
        .aliases(&["upgrade", "downgrade", "up", "down"])
        .short("Updates the Home Assistant Supervisor")
        .long(
            "
Using this command you can upgrade or downgrade the Home Assistant Supervisor
running on your Home Assistant  system to the latest version
or the version specified.",
        )
        .example(
            "
  ha supervisor update
  ha supervisor update --version 173",
        )
        .flag(version_flag())
        .run(run_update("supervisor"))
}
