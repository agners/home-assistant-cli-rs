use super::{mounts_completions, send};
use crate::client::{self, Options};
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("delete [name]")
        .aliases(&["del", "remove", "rm"])
        .short("Delete a mount from Supervisor")
        .long(
            "
Unmount and delete an existing mount from Supervisor.
",
        )
        .example(
            "
  ha mounts delete my_share
",
        )
        .valid_args(mounts_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);
            send(ctx, "DELETE", "{name}", client::json_request(), Options::new())
        })
}
