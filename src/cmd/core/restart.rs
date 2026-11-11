use serde_json::Value;

use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("restart")
        .aliases(&["reboot"])
        .short("Restarts the Home Assistant Core")
        .long(
            "
Restart the Home Assistant Core instance running on your system",
        )
        .example(
            "
  ha core restart",
        )
        .flag(
            Flag::bool("safe-mode", false, "Restart Home Assistant in safe mode")
                .short('s')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .flag(
            Flag::bool("force", false, "Force restart during an offline db migration")
                .short('f')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("core", "restart")
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .spinner()
                .run_with(|ctx, options| {
                    if ctx.bool("safe-mode") {
                        options.insert("safe_mode".to_string(), Value::Bool(true));
                    }
                    if ctx.bool("force") {
                        options.insert("force".to_string(), Value::Bool(true));
                    }
                }),
        )
}
