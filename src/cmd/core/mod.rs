//! `ha core` (Go: `cmd/core*.go`).

mod check;
mod info;
mod logs;
mod options;
mod rebuild;
mod restart;
mod start;
mod stats;
mod stop;
mod update;

use crate::cmd::helpers::warn_deprecated_aliases;
use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("core")
        .aliases(&["homeassistant", "home-assistant", "ha"])
        .short("Provides control of the Home Assistant Core")
        .long(
            "
This command provides a set of subcommands to control the Home Assistant Core
instance running on this installation.

It provides commands to control Home Assistant Core (start, stop, restart),
but also allows you to check your Home Assistant Core configuration.
Furthermore, some options can be set and allows for upgrading/downgrading
Home Assistant Core.
",
        )
        .example(
            "
  ha core check
  ha core restart
  ha core update
\tha core update --version 2021.11.5",
        )
        .persistent_pre_run(|ctx| {
            // Go only warns about these two; `home-assistant` is not deprecated.
            warn_deprecated_aliases(&["homeassistant", "ha"], "core");
            crate::cmd::root_pre_run(ctx);
        })
        .subcommand(check::command())
        .subcommand(info::command())
        .subcommand(logs::command())
        .subcommand(options::command())
        .subcommand(rebuild::command())
        .subcommand(restart::command())
        .subcommand(start::command())
        .subcommand(stats::command())
        .subcommand(stop::command())
        .subcommand(update::command())
}
