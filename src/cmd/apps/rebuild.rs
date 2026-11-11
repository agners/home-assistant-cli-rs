use serde_json::Value;

use crate::client;
use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, Flag};

pub fn command() -> Command {
    Command::new("rebuild [slug]")
        .aliases(&["rb", "reinstall"])
        .short("Rebuild a locally built Home Assistant app")
        .long(
            "
Most apps provide pre-built images Home Assistant can download and use.
However, some don't. This is usually the case for local or development version
of apps. This command allows you to trigger a rebuild of a locally built app.
",
        )
        .example(
            "
  ha apps rebuild local_my_app
  ha apps rebuild local_my_app --force
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .flag(Flag::bool(
            "force",
            false,
            "Force rebuild of the app even if pre-built images are provided",
        ))
        .run(
            Call::post("addons", "{slug}/rebuild")
                .arg_param("slug", 0)
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .spinner()
                .run_with(|ctx, options| {
                    if ctx.bool("force") {
                        options.insert("force".into(), Value::Bool(true));
                    }
                }),
        )
}
