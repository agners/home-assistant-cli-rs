use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reset")
        .short("Resets the internal Home Assistant DNS server configuration")
        .long("Resets the internal Home Assistant DNS server configuration.")
        .example("ha dns reset")
        .run(Call::post("dns", "reset").spinner().run())
}
