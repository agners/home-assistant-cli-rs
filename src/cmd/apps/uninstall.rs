use serde_json::Value;

use crate::client;
use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, Flag, comp};

pub fn command() -> Command {
    Command::new("uninstall [slug]")
        .aliases(&["remove", "delete", "del", "rem", "un", "uninst"])
        .short("Uninstalls a Home Assistant app")
        .long(
            "
This command allows you to uninstall a Home Assistant app.
",
        )
        .example(
            "
  ha apps uninstall core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .flag(
            Flag::bool("remove-config", false, "Delete app's config folder (if used)")
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("addons", "{slug}/uninstall")
                .arg_param("slug", 0)
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .run_with(|ctx, options| {
                    // Only sent when true (not merely when changed).
                    if ctx.bool("remove-config") {
                        options.insert("remove_config".into(), Value::Bool(true));
                    }
                }),
        )
}
