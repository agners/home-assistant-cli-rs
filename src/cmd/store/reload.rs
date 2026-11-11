use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reload")
        .aliases(&["refresh", "re"])
        .short("Reloads/Refreshes the Home Assistant app store")
        .long(
            "
This command allows you to force a reload/refresh of the Home Assistant app
store. Using this, you can force the download of the most recent version
information of an app. This might be helpful when you know a new version of
an app is released, but not yet available as an upgrade in Home Assistant.
",
        )
        .example(
            "
  ha store reload
",
        )
        .run(Call::post("store", "reload").spinner().run())
}
