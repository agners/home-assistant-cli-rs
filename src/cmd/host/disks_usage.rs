use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("usage")
        .aliases(&["us", "use"])
        .short("Get default disk usage information")
        .long(
            "
This command provides information about the default disk usage on the host system
that Home Assistant is running on.",
        )
        .example(
            "
  ha host disks usage",
        )
        .run(Call::get("host", "disks/default/usage").run())
}
