use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides information on the host system")
        .long(
            "
This command provides information on the host system Home Assistant is
running on",
        )
        .example(
            "
  ha host info",
        )
        .run(Call::get("host", "info").run())
}
