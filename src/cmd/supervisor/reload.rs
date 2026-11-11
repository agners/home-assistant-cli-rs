use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reload")
        .aliases(&["refresh", "re"])
        .short("Reload the Home Assistant Supervisor updating information")
        .long(
            "
Reloading the Home Assistant Supervisor, triggers the Supervisor to regather
all data it currently has, including checking for updates.",
        )
        .example(
            "
  ha supervisor reload",
        )
        .run(Call::post("supervisor", "reload").run())
}
