//! `ha store apps` (Go: `cmd/store_apps*.go`).

mod install;
mod update;

use crate::cmd::helpers::{Call, warn_deprecated_aliases};
use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("apps")
        .aliases(&["app", "addons", "add-on", "addon", "add-ons"])
        .short("Install and update Home Assistant apps")
        .long(
            "
The store command allows you to manage Home Assistant apps by exposing
commands for installing or update them.",
        )
        .example(
            "
  ha store apps install core_ssh
  ha store apps update core_ssh",
        )
        .persistent_pre_run(|ctx| {
            warn_deprecated_aliases(&["addons", "addon", "add-on", "add-ons"], "apps");
            crate::cmd::root_pre_run(ctx);
        })
        .run(Call::get("store", "addons").run())
        .subcommand(install::command())
        .subcommand(update::command())
}
