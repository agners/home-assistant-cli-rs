use crate::client;
use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("restart [slug]")
        .aliases(&["reboot"])
        .short("Restarts a Home Assistant app")
        .long(
            "
Restart a Home Assistant app
",
        )
        .example(
            "
  ha apps restart core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        // Note: no spinner here in the Go source.
        .run(
            Call::post("addons", "{slug}/restart")
                .arg_param("slug", 0)
                .timeout(client::CONTAINER_OPERATION_TIMEOUT)
                .run(),
        )
}
