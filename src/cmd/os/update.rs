use crate::client::{self, Options};
use crate::cmd::helpers::{Call, debug_args, leaf, string_if_set, version_flag};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("update")
        .aliases(&["upgrade", "downgrade", "up", "down"])
        .short("Updates the Home Assistant Operating System")
        // Note: the trailing space after "Home Assistant" is in the Go source.
        .long(
            "
Using this command you can upgrade or downgrade the Home Assistant\x20
Operating System to the latest version or the version specified.
",
        )
        .example(
            "
  ha os update
  ha os update --version 5
",
        )
        .flag(version_flag())
        .run(|ctx| {
            debug_args(ctx);

            let mut options = Options::new();
            string_if_set(ctx, &mut options, "version", "version");

            let success = Call::post("os", "update")
                .timeout(client::OS_DOWNLOAD_TIMEOUT)
                .spinner()
                .execute(ctx, options);
            if success {
                println!("\nOS update applied. Reboot the device using `ha host reboot` to finish the update.");
            }
            success
        })
}
