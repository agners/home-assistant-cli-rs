//! `ha resolution issue` (Go: `cmd/resolution_issue*.go`).

mod dismiss;
mod suggestions;

use super::info_completions;
use crate::cobra::{Command, Completer};

pub fn command() -> Command {
    Command::new("issue")
        .aliases(&["is", "trouble"])
        .short("Issues management reported by Resolution center")
        .long(
            "
This command allows dismissing issues reported by the system.",
        )
        .example(
            "
  ha resolution issue dismiss [id]",
        )
        .subcommand(dismiss::command())
        .subcommand(suggestions::command())
}

/// Go: `resolutionIssueCompletions` — issue UUIDs, described by their type.
fn issue_completions() -> Completer {
    info_completions("issues", "uuid", Some("type"))
}
