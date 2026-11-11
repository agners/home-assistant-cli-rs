//! `ha backups` — create, restore and remove backups (Go: `cmd/backups*.go`).

mod freeze;
mod info;
mod new;
mod options;
mod reload;
mod remove;
mod restore;
mod thaw;

use serde_json::Value;

use crate::cmd::completions::{self, api_data, describe};
use crate::cmd::helpers::{self, Call};
use crate::cobra::{Command, Completer, Ctx, Directive, comp};

/// Go: `backupsFolders`
const BACKUPS_FOLDERS: [&str; 4] = ["addons", "media", "share", "ssl"];

pub fn command() -> Command {
    Command::new("backups")
        .aliases(&[
            "backup",
            "back",
            "backups",
            "bk",
            "snapshots",
            "snapshot",
            "snap",
            "shot",
            "sn",
        ])
        .short("Create, restore and remove backups")
        .long(
            "
Backups of your Home Assistant system, which you can create,
restore, and delete using this command.",
        )
        .example(
            "
  ha backups
  ha backups new",
        )
        .persistent_pre_run(|ctx| {
            helpers::warn_deprecated_aliases(&["snapshots", "snapshot", "snap", "shot", "sn"], "backups");
            crate::cmd::root_pre_run(ctx);
        })
        .run(Call::get("backups", "info").run())
        .subcommand(freeze::command())
        .subcommand(info::command())
        .subcommand(new::command())
        .subcommand(options::command())
        .subcommand(reload::command())
        .subcommand(remove::command())
        .subcommand(restore::command())
        .subcommand(thaw::command())
}

/// `cmd.Flags().GetStringArray(name)`. pflag re-parses the flag's `[a,b]`
/// text form, which turns a single empty value (`--app ""`) into an empty
/// list; `Ctx::string_array` returns `[""]` for it.
fn string_array(ctx: &Ctx, name: &str) -> Vec<String> {
    let mut values = ctx.string_array(name);
    if values == [""] {
        values.clear();
    }
    values
}

/// Go: `backupsCompletions` — backup slugs with name, date and type.
fn backups_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = api_data("backups", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for backup in data.get("backups").and_then(Value::as_array).into_iter().flatten() {
            let Some(m) = backup.as_object() else { continue };
            let Some(slug) = m.get("slug").and_then(Value::as_str) else {
                continue;
            };
            ret.push(describe(slug, m, &["name", "date", "type"]));
        }
        (ret, Directive::NO_FILE_COMP)
    })
}

/// Go: `backupsAppsCompletions` — installed apps plus `ALL`.
fn backups_apps_completions() -> Completer {
    let apps = completions::apps();
    comp::from_fn(move |ctx, to_complete| {
        if !ctx.args.is_empty() {
            return (Vec::new(), Directive::NO_FILE_COMP);
        }
        let (mut ret, directive) = apps(ctx, to_complete);
        ret.push("ALL\tAll currently installed apps".to_string());
        (ret, directive)
    })
}

/// Go: `backupsFoldersCompletions`
fn backups_folders_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        if !ctx.args.is_empty() {
            return (Vec::new(), Directive::NO_FILE_COMP);
        }
        (
            BACKUPS_FOLDERS.iter().map(|s| s.to_string()).collect(),
            Directive::NO_FILE_COMP,
        )
    })
}
