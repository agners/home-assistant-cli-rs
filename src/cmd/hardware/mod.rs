//! `ha hardware` — hardware information (Go: `cmd/hardware*.go`).

mod audio;
mod info;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("hardware")
        .aliases(&["hw"])
        .short("Provides hardware information about your system")
        .long(
            "
The hardware command provides information about the hardware of your system
that is running Home Assistant. It is useful for finding things like: available
audio devices and serial ports.",
        )
        .example(
            "
  ha hardware info
  ha hardware audio",
        )
        .subcommand(audio::command())
        .subcommand(info::command())
}
