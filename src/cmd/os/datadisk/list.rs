use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("list")
        .aliases(&["in", "inf", "info", "show"])
        .short("Provides information about the running Home Assistant Operating System")
        .long(
            "
This command provides general information about available Harddisk for using with Home Assistant Operating System.
",
        )
        .example(
            "
  ha os datadisk list
",
        )
        .run(Call::get("os", "datadisk/list").run())
}
