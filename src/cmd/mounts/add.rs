use serde_json::Value;

use super::{add_mount_flags, mount_flags_to_new_options, mounts_completions, send};
use crate::client::{self, Options};
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    let cmd = Command::new("add [name]")
        .aliases(&["create", "new"])
        .short("Add new mount to Supervisor")
        .long(
            "
Add and configure a new mount in Supervisor.
",
        )
        .example(
            "
  ha mounts add my_share --usage media --type cifs --server server.local --share media
  ha mounts add media_disk --usage media --type disk --device /dev/sda1
",
        )
        .valid_args(mounts_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);

            let request = client::json_request_timeout(client::MOUNT_TIMEOUT);

            let mut options = Options::new();
            options.insert("name".to_string(), Value::from(ctx.arg(0)));
            mount_flags_to_new_options(ctx, &mut options);

            send(ctx, "POST", "", request, options)
        });
    add_mount_flags(cmd, "cifs", "media", false)
}
