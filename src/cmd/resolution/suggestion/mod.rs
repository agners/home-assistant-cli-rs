//! `ha resolution suggestion` (Go: `cmd/resolution_suggestion*.go`).

mod apply;
mod dismiss;

use super::info_completions;
use crate::cobra::{Command, Completer};

pub fn command() -> Command {
    Command::new("suggestion")
        .aliases(&["su", "solution"])
        .short("Suggestion management reported by Resolution center")
        .long(
            "
This command allows to dismiss or apply suggestion reported by the system.",
        )
        .example(
            "
  ha resolution suggestion dismiss [id]
  ha resolution suggestion apply [id]",
        )
        .subcommand(apply::command())
        .subcommand(dismiss::command())
}

/// Go: `resolutionSuggestionCompletions` — suggestion UUIDs, described by
/// their type.
fn suggestion_completions() -> Completer {
    info_completions("suggestions", "uuid", Some("type"))
}
