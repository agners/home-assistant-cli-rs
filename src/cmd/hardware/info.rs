use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("info")
        .aliases(&["in", "inf"])
        .short("Provides hardware information about your system")
        // Note: the trailing space after "available" is in the Go source.
        .long(
            "
The hardware command provides information about the hardware of your system
that is running Home Assistant. It is useful for finding things like: available\x20
serial ports.",
        )
        .example("ha hardware info")
        .run(Call::get("hardware", "info").run())
}
