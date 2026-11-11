use crate::cmd::helpers::{leaf, run_update, version_flag};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("update")
        .aliases(&["upgrade", "downgrade", "up", "down"])
        .short("Updates the internal Home Assistant DNS server")
        // Note: the trailing space after "Home Assistant" is in the Go source.
        .long(
            "
Using this command you can upgrade or downgrade the internal Home Assistant\x20
DNS server, to the latest version or the version specified.
",
        )
        .example(
            "
  ha dns update --version 5
",
        )
        .flag(version_flag())
        .run(run_update("dns"))
}
