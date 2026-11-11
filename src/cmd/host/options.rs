use crate::cmd::helpers::{Call, leaf, string_if_set};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options on host system")
        // Note: the trailing space after "that" is in the Go source.
        .long(
            "
This command allows you to set configuration options on the host system that\x20
your Home Assistant is running on.",
        )
        .example(
            "
  ha host options --hostname homeassistant.local",
        )
        .flag(Flag::string("hostname", "", "Hostname to set").completion(comp::no_file()))
        .run(Call::post("host", "options").run_with(|ctx, options| string_if_set(ctx, options, "hostname", "hostname")))
}
