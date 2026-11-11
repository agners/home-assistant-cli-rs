use crate::cmd::helpers::{leaf, run_update, version_flag};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("update")
        .aliases(&["upgrade", "downgrade", "up", "down"])
        .short("Updates the Home Assistant Audio")
        .long(
            "
Using this command you can upgrade or downgrade the Home Assistant Audio
instance running on your system to the latest version or the version specified.",
        )
        .example(
            "
  ha audio update
  ha audio update --version 6",
        )
        .flag(version_flag())
        .run(run_update("audio"))
}
