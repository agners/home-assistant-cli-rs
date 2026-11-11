//! `ha resolution` — Resolution center (Go: `cmd/resolution*.go`).

mod check;
mod healthcheck;
mod info;
mod issue;
mod suggestion;

use serde_json::Value;

use crate::cmd::completions::api_data;
use crate::cobra::{Command, Completer, Directive, comp};

pub fn command() -> Command {
    Command::new("resolution")
        .aliases(&["resolutions", "res"])
        .short("Resolution center of Supervisor, show issues and suggest solutions")
        // Note: the trailing space after "suggestions." is in the Go source.
        .long(
            "
The Resolution center provides information about detected issues on the system.
It also gives suggestions that can fix the issue and can be executed by the suggestion command.
It is possible to dismiss issues or suggestions.\x20
If you are running an unsupported system, the reasons for it will also show here",
        )
        .example(
            "
  ha resolution info
  ha resolution suggestion apply [id]",
        )
        .subcommand(check::command())
        .subcommand(healthcheck::command())
        .subcommand(info::command())
        .subcommand(issue::command())
        .subcommand(suggestion::command())
}

/// The shape shared by `resolutionCheckCompletions`,
/// `resolutionIssueCompletions` and `resolutionSuggestionCompletions`: the
/// string field `id_key` of every object in the list `list_key` of
/// `resolution/info`, described by its string field `desc_key` (if any).
fn info_completions(list_key: &'static str, id_key: &'static str, desc_key: Option<&'static str>) -> Completer {
    comp::from_fn(move |ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = api_data("resolution", "info") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for item in data.get(list_key).and_then(Value::as_array).into_iter().flatten() {
            let Some(m) = item.as_object() else { continue };
            let Some(id) = m.get(id_key).and_then(Value::as_str) else {
                continue;
            };
            let mut s = id.to_string();
            if let Some(t) = desc_key.and_then(|key| m.get(key)).and_then(Value::as_str) {
                s.push('\t');
                s.push_str(t);
            }
            ret.push(s);
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
