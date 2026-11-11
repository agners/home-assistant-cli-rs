//! `ha audio default` (Go: `cmd/audio_default*.go`).

mod input;
mod output;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("default")
        .aliases(&["def", "de", "standard", "std"])
        .short("Set default input/output audio device.")
        .long(
            "
Set the default input/output audio device of your Home Assistant system.
",
        )
        .example(
            "
\tha audio default input --name \"...\"
\tha audio default output --name \"...\"
",
        )
        .subcommand(input::command())
        .subcommand(output::command())
}
