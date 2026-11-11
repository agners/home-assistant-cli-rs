use crate::client;
use crate::cmd::helpers::{Call, bool_if_changed, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options for the internal Home Assistant Multicast server")
        .long(
            "
This command allows you to set configuration options for the internally
running Home Assistant Multicast server.
",
        )
        .example(
            "
  ha multicast options --enabled=false
  ha multicast options --enabled
",
        )
        .flag(
            Flag::bool("enabled", true, "Enable/Disable the Multicast plug-in")
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("multicast", "options")
                .timeout(client::CONTAINER_DOWNLOAD_TIMEOUT)
                .spinner()
                .run_with(|ctx, options| bool_if_changed(ctx, options, "enabled", "enabled")),
        )
}
