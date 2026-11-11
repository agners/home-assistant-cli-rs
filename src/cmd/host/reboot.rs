use serde_json::Value;

use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("reboot")
        .aliases(&["restart", "rb"])
        .short("Reboots the host machine")
        .long(
            "
Reboot the machine that your Home Assistant is running on.",
        )
        .example(
            "
  ha host reboot",
        )
        .flag(
            Flag::bool("force", false, "Force reboot during an offline db migration")
                .short('f')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("host", "reboot")
                .timeout(client::REBOOT_TIMEOUT)
                .run_with(|ctx, options| {
                    if ctx.bool("force") {
                        options.insert("force".into(), Value::Bool(true));
                    }
                }),
        )
}
