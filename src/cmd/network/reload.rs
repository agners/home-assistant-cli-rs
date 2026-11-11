use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reload")
        .aliases(&["re"])
        .short("Reload Network information the host")
        .long(
            "
Reload information about the host network and interfaces.
",
        )
        .example(
            "
  ha network reload
",
        )
        .run(Call::post("network", "reload").run())
}
