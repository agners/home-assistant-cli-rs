use crate::client::{self, Options};
use crate::cmd::helpers::{Call, debug_args, leaf};
use crate::cobra::Command;
use crate::prompt;

pub fn command() -> Command {
    leaf("reset-storage")
        .short("Reset the Docker storage")
        .long(
            "
This command wipes the complete Docker storage on the next reboot. All container
images, including Supervisor, Home Assistant Core, plugins and apps, are deleted
and downloaded again once the system is back up.

Use it to recover from a corrupted or inconsistent Docker storage, for example
when containers fail to start or images cannot be updated.

Configuration and data of Home Assistant and apps are kept, they are stored
outside of the Docker storage.

Internet connectivity is required for re-download of all the container images.
A reboot is required to apply the reset.
",
        )
        .example(
            "
  ha docker reset-storage
",
        )
        .run(|ctx| {
            debug_args(ctx);

            let confirmed = match prompt::ask_for_confirmation(
                "
This will delete all container images on the next reboot. Supervisor, Home
Assistant Core, plugins and apps are downloaded again afterwards, which
requires internet connectivity. Home Assistant and app data are kept.

Once confirmed, the reset will be applied on the next system reboot.

Are you sure you want to proceed?",
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

            if !Call::post("docker", "reset-storage").execute(ctx, Options::new()) {
                return false;
            }
            if !client::raw_json() {
                println!(
                    "\nDocker storage reset scheduled. Reboot the device using `ha host reboot` to apply the reset."
                );
            }
            true
        })
}
