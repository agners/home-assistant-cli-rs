use super::{add_mount_flags, mount_flags_to_options, mounts_completions, send};
use crate::client::{self, Options};
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    let cmd = Command::new("update [name]")
        .aliases(&["change", "set", "up", "modify", "mod"])
        .short("Update configuration of a mount in Supervisor")
        .long(
            "
Update or change the configuration of an existing mount in Supervisor.
The new configuration replaces the old one, so pass every option the mount needs.
",
        )
        .example(
            "
  ha mounts update my_share --usage media --type cifs --server server.local --share media
  ha mounts update media_disk --usage media --type disk --device /dev/sda1 --read-only
",
        )
        .valid_args(mounts_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);

            let request = client::json_request_timeout(client::MOUNT_TIMEOUT);

            let mut options = Options::new();
            mount_flags_to_options(ctx, &mut options);

            send(ctx, "PUT", "{name}", request, options)
        });
    // Go clears the displayed defaults of --type / --usage (`DefValue = ""`)
    // and marks both required. The real defaults ("cifs", "media") are never
    // observable: only changed flags are sent and both must be given.
    add_mount_flags(cmd, "", "", true)
}
