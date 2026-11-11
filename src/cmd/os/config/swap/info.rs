use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "info"])
        .short("Show HAOS swap settings")
        .long(
            "
This command allows you to see how swap is used by the Home Assistant OS.",
        )
        .example(
            "
  ha os config swap info",
        )
        .run(Call::get("os", "config/swap").run())
}
