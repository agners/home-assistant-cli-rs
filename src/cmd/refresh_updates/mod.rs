//! `ha refresh-updates` (Go: `cmd/refresh_updates.go`).

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("refresh-updates")
        .aliases(&["refresh", "refresh_updates"])
        .short("Reload stores and version information")
        .long(
            "
This command reloads information about app repositories and fetches new version files.
\t",
        )
        .example(
            "
  ha refresh-updates
\t",
        )
        .run(Call::post("refresh_updates", "").run())
}
