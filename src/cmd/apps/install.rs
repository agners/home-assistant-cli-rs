use crate::client;
use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("install [slug]")
        .aliases(&["i", "inst"])
        .short("Installs a Home Assistant app")
        .long(
            "
This command allows you to install a Home Assistant app from the commandline.
",
        )
        .example(
            "
  ha apps install core_ssh
",
        )
        .valid_args(completions::store_apps())
        .args(Args::Exact(1))
        .run(
            Call::post("addons", "{slug}/install")
                .arg_param("slug", 0)
                .timeout(client::CONTAINER_DOWNLOAD_TIMEOUT)
                .spinner()
                .run(),
        )
}
