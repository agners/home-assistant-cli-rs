//! `ha store` — app store and repository management (Go: `cmd/store*.go`).

mod apps;
mod reload;
mod repositories_add;
mod repositories_delete;
mod repositories_repair;

use serde_json::Value;

use crate::cmd::completions::{api_data, describe};
use crate::cmd::helpers::Call;
use crate::cobra::{Command, Completer, Directive, comp};

pub fn command() -> Command {
    Command::new("store")
        .aliases(&["shop", "stor"])
        .short("Install and update Home Assistant apps and manage stores")
        .long(
            "
The store command allows you to manage Home Assistant apps by exposing
commands for installing or updating them. It also provides functionality
for managing stores that provide additional apps.",
        )
        .example(
            "
  ha store apps install core_ssh
  ha store add https://github.com/home-assistant/addons-example
  ha store delete 94cfad5a
  ha store reload",
        )
        .run(Call::get("store", "").run())
        .subcommand(apps::command())
        .subcommand(reload::command())
        .subcommand(repositories_add::command())
        .subcommand(repositories_delete::command())
        .subcommand(repositories_repair::command())
}

/// Go: `storeRepositoriesCompletions` — slugs of the configured repositories.
fn repositories_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = api_data("store", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for repo in data.get("repositories").and_then(Value::as_array).into_iter().flatten() {
            let Some(m) = repo.as_object() else { continue };
            let Some(slug) = m.get("slug").and_then(Value::as_str) else {
                continue;
            };
            ret.push(describe(slug, m, &["name", "url"]));
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
