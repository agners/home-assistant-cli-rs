use crate::client;
use crate::cmd::helpers::{Call, bool_if_changed, leaf, string_if_set, version_flag};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("update")
        .aliases(&["upgrade", "downgrade", "up", "down"])
        .short("Updates the Home Assistant Core")
        .long(
            "
Using this command you can upgrade or downgrade the Home Assistant Core instance
running on your system to the latest version or the version specified.",
        )
        .example(
            "
  ha core update
  ha core update --version 0.105.4",
        )
        .flag(version_flag())
        .flag(Flag::bool("backup", false, "Create partial backup before update").completion(comp::bools()))
        .run(
            Call::post("core", "update")
                .timeout(client::CONTAINER_DOWNLOAD_TIMEOUT)
                .spinner()
                .run_with(|ctx, options| {
                    string_if_set(ctx, options, "version", "version");
                    bool_if_changed(ctx, options, "backup", "backup");
                }),
        )
}
