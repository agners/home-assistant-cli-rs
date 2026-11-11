use super::{mounts_completions, send};
use crate::client::{self, Options};
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("reload [name]")
        .aliases(&["re", "refresh", "remount"])
        .short("Reload a mount in Supervisor")
        .long(
            "
Unmount and remount an existing mount in Supervisor using
the same configuration.
",
        )
        .example(
            "
  ha mounts reload my_share
",
        )
        .valid_args(mounts_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);
            let request = client::json_request_timeout(client::MOUNT_TIMEOUT);
            send(ctx, "POST", "{name}/reload", request, Options::new())
        })
}
