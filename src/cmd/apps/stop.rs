use crate::client;
use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("stop [slug]")
        .aliases(&["halt", "shutdown", "quit"])
        .short("Manually stop a running Home Assistant app")
        .long(
            "
This command allows you to manually stop a Home Assistant app
",
        )
        .example(
            "
  ha apps stop core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .run(
            Call::post("addons", "{slug}/stop")
                .arg_param("slug", 0)
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .run(),
        )
}
