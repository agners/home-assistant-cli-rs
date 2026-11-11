use serde_json::Value;

use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("shutdown")
        .aliases(&["sh"])
        .short("Shutdown the host machine")
        .long(
            "
Shuts down the machine that your Home Assistant is running on.
WARNING: This is turning off the computer/device.",
        )
        .example(
            "
  ha host shutdown",
        )
        .flag(
            Flag::bool("force", false, "Force shutdown during an offline db migration")
                .short('f')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("host", "shutdown")
                .timeout(client::REBOOT_TIMEOUT)
                .run_with(|ctx, options| {
                    if ctx.bool("force") {
                        options.insert("force".into(), Value::Bool(true));
                    }
                }),
        )
}
