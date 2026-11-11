use serde_json::Value;

use crate::client;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("stop")
        .short("Manually stop Home Assistant Core")
        .long(
            "
This command allows you to manually stop the Home Assistant Core instance on
your system.",
        )
        .example(
            "
  ha core stop",
        )
        .flag(
            Flag::bool("force", false, "Force stop during an offline db migration")
                .short('f')
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("core", "stop")
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .spinner()
                .run_with(|ctx, options| {
                    if ctx.bool("force") {
                        options.insert("force".to_string(), Value::Bool(true));
                    }
                }),
        )
}
