//! `ha authentication` — Home Assistant user accounts (Go: `cmd/auth*.go`).

mod cache;
mod list;
mod reset;

use crate::cobra::{Command, Ctx};
use crate::{debug, gofmt};

/// The Go commands log as `auth <name>`, not with the command path.
fn debug_args(ctx: &Ctx) {
    debug!(&format!("auth {}", ctx.name()), "args" => gofmt::list(&ctx.args));
}

pub fn command() -> Command {
    Command::new("authentication")
        .aliases(&["auth", "au"])
        .short("Authentication for Home Assistant users.")
        .long(
            "
The authentication command allows you to manage Home Assistant user accounts.
",
        )
        .example(
            "
  ha authentication reset --username \"JohnDoe\" --password \"123SuperSecret!\"
\t",
        )
        .subcommand(cache::command())
        .subcommand(list::command())
        .subcommand(reset::command())
}
