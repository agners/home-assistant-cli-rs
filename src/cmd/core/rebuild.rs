use serde_json::Value;

use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("rebuild")
        .aliases(&["rb", "reinstall"])
        .short("Rebuild the Home Assistant Core instance")
        .long(
            "
This command allows you to trigger a rebuild for your Home Assistant Core
instance running on your Home Assistant system.
Don't worry, this does not delete your config.",
        )
        .example(
            "
  ha core rebuild",
        )
        .flag(
            Flag::bool("safe-mode", false, "Rebuild Home Assistant in safe mode")
                .short('s')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .flag(
            Flag::bool("force", false, "Force rebuild during an offline db migration")
                .short('f')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("core", "rebuild")
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
