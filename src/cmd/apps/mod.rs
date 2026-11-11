//! `ha apps` — app (add-on) management (Go: `cmd/apps*.go`).

mod changelog;
mod info;
mod install;
mod logs;
mod rebuild;
mod restart;
mod start;
mod stats;
mod stop;
mod uninstall;
pub mod update;

use crate::cmd::helpers::{Call, warn_deprecated_aliases};
use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("apps")
        .aliases(&["app", "addons", "addon", "add-on", "add-ons", "ad"])
        .short("Install, update, remove and configure Home Assistant apps")
        .long(
            "
The apps command allows you to manage Home Assistant apps by exposing
commands for installing, removing, configure and control them. It also provides
information commands for apps.",
        )
        .example(
            "
  ha apps logs core_ssh
  ha apps install core_ssh
  ha apps start core_ssh",
        )
        .persistent_pre_run(|ctx| {
            warn_deprecated_aliases(&["addons", "addon", "add-on", "add-ons", "ad"], "apps");
            crate::cmd::root_pre_run(ctx);
        })
        .run(Call::get("addons", "").run())
        .subcommand(changelog::command())
        .subcommand(info::command())
        .subcommand(install::command())
        .subcommand(logs::command())
        .subcommand(rebuild::command())
        .subcommand(restart::command())
        .subcommand(start::command())
        .subcommand(stats::command())
        .subcommand(stop::command())
        .subcommand(uninstall::command())
        .subcommand(update::command())
}
