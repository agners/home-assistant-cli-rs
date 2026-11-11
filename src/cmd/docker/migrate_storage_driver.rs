use serde_json::Value;

use crate::client::{self, Options};
use crate::cmd::helpers::{Call, debug_args};
use crate::cobra::{Args, Command, Directive, comp};
use crate::prompt;

pub fn command() -> Command {
    Command::new("migrate-storage-driver [driver]")
        .short("Migrate Docker storage driver")
        .long(
            "
This command schedules a Docker storage driver migration to be performed on the
next reboot. By default, it migrates to the (Containerd snapshotter) overlayfs.

The migration will be applied during the next system reboot. A reboot is required
to complete the migration.
",
        )
        .example(
            "
  ha docker migrate-storage-driver
  ha docker migrate-storage-driver overlayfs
",
        )
        .valid_args(comp::from_fn(|ctx, _| {
            if !ctx.args.is_empty() {
                return (Vec::new(), Directive::NO_FILE_COMP);
            }
            (vec!["overlayfs".to_string()], Directive::NO_FILE_COMP)
        }))
        .args(Args::Maximum(1))
        .run(|ctx| {
            debug_args(ctx);

            // Default to overlayfs if no argument provided
            let storage_driver = if ctx.args.is_empty() {
                "overlayfs"
            } else {
                ctx.arg(0)
            };

            let confirmed = match prompt::ask_for_confirmation(
                &format!(
                    "
This will schedule a Docker storage driver migration to \"{storage_driver}\".
Make sure to create a full Home Assistant backup before proceeding.

Internet connectivity is required for re-download of all the container images
and it is recommended to have at least 50% of free storage.

Once confirmed, the migration will be applied on the next system reboot.
Are you sure you want to proceed?"
                ),
                0,
            ) {
                Ok(confirmed) => confirmed,
                Err(err) => {
                    eprintln!("Aborted: {err}");
                    return false;
                }
            };

            if !confirmed {
                eprintln!("Aborted.");
                return false;
            }

            let mut options = Options::new();
            options.insert("storage_driver".into(), Value::from(storage_driver));
            if !Call::post("docker", "migrate-storage-driver").execute(ctx, options) {
                return false;
            }
            if !client::raw_json() {
                println!(
                    "\nDocker storage driver migration scheduled. Reboot the device using `ha host reboot` to apply the migration."
                );
            }
            true
        })
}
