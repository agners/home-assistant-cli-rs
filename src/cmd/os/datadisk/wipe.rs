//! `ha os datadisk wipe` (Go: `cmd/os_datadisk_wipe.go`). `ha factory-reset`
//! reuses [`LONG`] and [`run`].

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Ctx};
use crate::{debug, gofmt, prompt};

/// `osDataDiskWipeCmd.Long`
pub const LONG: &str = "
This command will wipe all config/settings for apps, Home Assistant and the Operating
System and any locally stored data in config, backups, media, etc. The machine will
reboot during this.

After the reboot completes the latest stable version of Home Assistant and Supervisor
will be downloaded. Once the process is complete you will see onboarding, like
during initial setup.

This wipe also include network settings. So after the reboot you may need to reconfigure
those in order to access Home Assistant again.

Please note, this command is limited due to security reasons, and will
only work on some locations. For example, the Operating System CLI.
";

/// `osDataDiskWipeCmd.Run`: ask for confirmation, then POST `os/datadisk/wipe`.
pub fn run(ctx: &Ctx) -> bool {
    // Not `debug_args`: `ha factory-reset` logs the same message in Go.
    debug!("os datadisk wipe", "args" => gofmt::list(&ctx.args));

    let confirmed = prompt::ask_for_confirmation(
        "
This will completely wipe the datadisk. This process is irreversible.
Are you sure you want to proceed?",
        0,
    );

    match confirmed {
        Err(err) => {
            eprintln!("Aborted: {err}");
            false
        }
        Ok(true) => Call::post("os", "datadisk/wipe").execute(ctx, Default::default()),
        Ok(false) => {
            eprintln!("Aborted.");
            false
        }
    }
}

pub fn command() -> Command {
    leaf("wipe")
        .aliases(&["wipe", "reset", "erase"])
        .short("Wipe the Home Assistant Operating-System data partition")
        .long(LONG)
        .example(
            "
  ha os datadisk wipe
",
        )
        .run(run)
}
