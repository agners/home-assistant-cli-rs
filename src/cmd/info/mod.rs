//! `ha info` — general Home Assistant information (Go: `cmd/info.go`).
//!
//! A top-level command that is a leaf itself (no sub commands).

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides a general Home Assistant information overview")
        .long(
            "
This command provide a general information about your Home Assistant system.
The information provide can be useful for sharing when you are encountering
issues or when reporting one on GitHub.
\t",
        )
        .example(
            "
  ha info
\t",
        )
        .run(Call::get("info", "").run())
}
