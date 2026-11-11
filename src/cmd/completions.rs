//! Dynamic completers that are used by more than one command group. Group
//! specific completers live in their group's module.

use serde_json::Value;

use crate::client::{self, Options};
use crate::cobra::{Completer, Directive, comp};

/// GET `section/command` for completion purposes: the `data` object of a
/// successful (`result: ok`, HTTP 2xx) response, `None` otherwise.
pub fn api_data(section: &str, command: &str) -> Option<Options> {
    let resp = client::generic_json_get(section, command).ok()?;
    if !resp.is_success() || resp.data.result != "ok" {
        return None;
    }
    Some(resp.data.data)
}

/// `"value\tdesc1, desc2"`: append the non-empty string fields `keys` of `m`
/// as the candidate's description.
pub fn describe(value: &str, m: &Options, keys: &[&str]) -> String {
    let ds: Vec<&str> = keys
        .iter()
        .filter_map(|k| m.get(*k)?.as_str())
        .filter(|s| !s.is_empty())
        .collect();
    if ds.is_empty() {
        value.to_string()
    } else {
        format!("{value}\t{}", ds.join(", "))
    }
}

fn field_is(m: &Options, key: &str, want: &Value) -> bool {
    m.get(key) == Some(want)
}

/// Go: `appsCompletions` — installed app slugs, filtered by what makes sense
/// for the command being completed (`rebuild`, `start`, `stop`, `update`).
pub fn apps() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = api_data("addons", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for addon in data.get("addons").and_then(Value::as_array).into_iter().flatten() {
            let Some(m) = addon.as_object() else { continue };
            let Some(slug) = m.get("slug").and_then(Value::as_str) else {
                continue;
            };
            let skip = match ctx.name() {
                "rebuild" => field_is(m, "build", &Value::Bool(false)),
                "start" => field_is(m, "state", &Value::from("started")),
                "stop" => field_is(m, "state", &Value::from("stopped")),
                "update" => field_is(m, "update_available", &Value::Bool(false)),
                _ => false,
            };
            if !skip {
                ret.push(describe(slug, m, &["name", "url"]));
            }
        }
        (ret, Directive::NO_FILE_COMP)
    })
}

/// Go: `storeAppCompletions` — store app slugs, filtered for `install` and
/// `update`.
pub fn store_apps() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = api_data("store", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        let (yes, no) = (Value::Bool(true), Value::Bool(false));
        for addon in data.get("addons").and_then(Value::as_array).into_iter().flatten() {
            let Some(m) = addon.as_object() else { continue };
            let Some(slug) = m.get("slug").and_then(Value::as_str) else {
                continue;
            };
            let skip = match ctx.name() {
                "install" => field_is(m, "available", &no) || field_is(m, "installed", &yes),
                "update" => {
                    field_is(m, "available", &no)
                        || field_is(m, "installed", &no)
                        || field_is(m, "update_available", &no)
                }
                _ => false,
            };
            if !skip {
                ret.push(describe(slug, m, &["name", "url"]));
            }
        }
        (ret, Directive::NO_FILE_COMP)
    })
}

/// Go: `backupsLocationsCompletions` — `.local` plus active backup mounts.
pub fn backups_locations() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Ok(resp) = client::generic_json_get("mounts", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        if !resp.is_success() {
            return (ret, Directive::NO_FILE_COMP);
        }
        ret.push(".local\tLocal storage, /backups".to_string());
        if resp.data.result != "ok" {
            return (ret, Directive::NO_FILE_COMP);
        }
        for mount in resp
            .data
            .data
            .get("mounts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(m) = mount.as_object() else { continue };
            if !field_is(m, "usage", &Value::from("backup")) || !field_is(m, "state", &Value::from("active")) {
                continue;
            }
            let Some(name) = m.get("name").and_then(Value::as_str) else {
                continue;
            };
            ret.push(describe(name, m, &["server", "share", "path", "uuid", "filesystem"]));
        }
        (ret, Directive::NO_FILE_COMP)
    })
}

/// Go: `hostBootCompletions` — boot IDs for the `--boot` flag of the logs
/// commands. (Go iterates a map in random order; here the order is sorted.)
pub fn host_boot() -> Completer {
    comp::from_fn(|ctx, to_complete| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = api_data("host/logs/boots", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for (boot_id, boot_name) in data.get("boots").and_then(Value::as_object).into_iter().flatten() {
            let Some(s) = boot_name.as_str() else { continue };
            if to_complete.is_empty() || s.starts_with(to_complete) {
                ret.push(format!("{s}\tboot offset {boot_id}"));
            }
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
