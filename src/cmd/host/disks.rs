use super::disks_usage;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("disks")
        .aliases(&["disk"])
        .short("Get information about host disks and their usage")
        .long(
            "
The disks command lists the disks of the host system that Home Assistant is
running on, including the partitions which can be used for disk mounts. It also
provides access to other disk-related operations.",
        )
        .example(
            "
  ha host disks
  ha host disks usage",
        )
        .run(Call::get("host", "disks").run())
        .subcommand(disks_usage::command())
}
