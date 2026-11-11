//! `ha resolution check` (Go: `cmd/resolution_check*.go`).

mod options;
mod run;

use super::info_completions;
use crate::cobra::{Command, Completer};

pub fn command() -> Command {
    Command::new("check")
        .aliases(&["checks", "test", "che", "ch"])
        .short("Check management by the Resolution center")
        .long(
            "
This command allows to manage checks that are run by the system.",
        )
        .example(
            "
  ha resolution check options [slug]",
        )
        .subcommand(options::command())
        .subcommand(run::command())
}

/// Go: `resolutionCheckCompletions` — the slugs of the checks.
fn check_completions() -> Completer {
    info_completions("checks", "slug", None)
}
