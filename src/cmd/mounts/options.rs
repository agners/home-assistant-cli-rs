use serde_json::Value;

use crate::cmd::completions;
use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{self, Command, Flag};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Set options for mount manager")
        .long(
            "
Change value for options of mount manager in Supervisor.
",
        )
        .example(
            "
  ha mounts options --default-backup-mount my_share
",
        )
        // Go sets `NoOptDefVal = ""`, which is pflag's "value required" default.
        .flag(
            Flag::string("default-backup-mount", "", "Set default backup mount")
                .completion(completions::backups_locations()),
        )
        // backwards compatibility
        .normalize_flags(cobra::normalize_underscores)
        .run(Call::post("mounts", "options").run_with(|ctx, options| {
            if ctx.changed("default-backup-mount") {
                let backup_mount = ctx.string("default-backup-mount");
                let value = if backup_mount.is_empty() {
                    Value::Null
                } else {
                    Value::String(backup_mount)
                };
                options.insert("default_backup_mount".to_string(), value);
            }
        }))
}
