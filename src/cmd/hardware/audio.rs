use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("audio")
        .aliases(&["sounds", "snd", "au"])
        .short("Provides information about audio devices on your system")
        .long(
            "
The command provides information about audio devices available on your system.",
        )
        .example("ha hardware audio")
        .run(Call::get("hardware", "audio").run())
}
