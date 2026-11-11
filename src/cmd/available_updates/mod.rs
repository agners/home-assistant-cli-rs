//! `ha available-updates` (Go: `cmd/available_updates.go`).

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("available-updates")
        .aliases(&["updates", "available_updates"])
        .short("Provides information about current pending updates")
        .long(
            "
This command provides information about the currently pending updates on the system.
\t",
        )
        .example(
            "
  ha available-updates
\t",
        )
        .run(Call::get("available_updates", "").run())
}
