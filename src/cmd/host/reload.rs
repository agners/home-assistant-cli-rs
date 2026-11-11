use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reload")
        .aliases(&["update", "refresh", "re"])
        .short("Reload information from the host machine")
        .long(
            "
This commands reload the information Home Assistant has on the hostmachine.
If some setting are changed outside of Home Assistant, this commands updates
the internals of Home Assistant.",
        )
        .example(
            "
  ha host reload",
        )
        .run(Call::post("host", "reload").run())
}
