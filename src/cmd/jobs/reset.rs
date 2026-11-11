use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reset")
        .short("Resets the internal Home Assistant Job Manager configuration")
        .long("Resets the internal Home Assistant Job Manager configuration.")
        .example("ha jobs reset")
        .run(Call::post("jobs", "reset").spinner().run())
}
