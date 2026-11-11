//! `ha os` — Home Assistant Operating System management (Go: `cmd/os*.go`).

mod boards;
mod boot_slot;
mod config;
pub mod datadisk;
mod import;
mod info;
mod update;

use serde_json::Value;

use crate::client::{self, Options};
use crate::cmd::helpers::warn_deprecated_aliases;
use crate::cobra::{Command, Completer, Directive, comp};

pub fn command() -> Command {
    Command::new("os")
        .aliases(&["hassos"])
        .short("Operating System specific for updating, info and configuration imports")
        .long(
            "
This command set is specifically designed for the Home Assistant Operating System
and only works on those systems. It provides an interface to get information
about your Home Assistant Operating System, but also provides command to
upgrade the operating system and the operating system CLI. Finally,
it provides a command to import configurations from a USB stick.",
        )
        .example(
            "
  ha os info
  ha os update",
        )
        .persistent_pre_run(|ctx| {
            warn_deprecated_aliases(&["hassos"], "os");
            crate::cmd::root_pre_run(ctx);
        })
        .subcommand(boards::command())
        .subcommand(boot_slot::command())
        .subcommand(config::command())
        .subcommand(datadisk::command())
        .subcommand(import::command())
        .subcommand(info::command())
        .subcommand(update::command())
}

/// Go: `osGetBootSlots` — the `boot_slots` object of `os/info`.
/// `Ok(None)` if the request was answered but there are no boot slots.
fn get_boot_slots() -> client::Result<Option<Options>> {
    let resp = client::generic_json_get("os", "info")?;
    if !resp.is_success() {
        return Ok(None);
    }
    if resp.data.result == "ok"
        && let Some(Value::Object(boot_slots)) = resp.data.data.get("boot_slots")
    {
        return Ok(Some(boot_slots.clone()));
    }
    Ok(None)
}

/// Go: `osBootSlotCompletions` — `other` and the name of the inactive slot.
/// (Go iterates the slots in random map order; here the order is sorted.)
fn boot_slot_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        if !ctx.args.is_empty() {
            return (Vec::new(), Directive::NO_FILE_COMP);
        }
        if let Ok(Some(boot_slots)) = get_boot_slots() {
            for (boot_slot, v) in &boot_slots {
                let Some(info) = v.as_object() else { continue };
                match info.get("state").and_then(Value::as_str) {
                    None | Some("active") => continue,
                    Some(_) => {}
                }
                let mut ret = vec!["other".to_string(), boot_slot.clone()];
                let details: Vec<&str> = ["version", "status"]
                    .iter()
                    .filter_map(|key| info.get(*key)?.as_str())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !details.is_empty() {
                    let details = format!("\t{}", details.join(", "));
                    ret[0] += &details;
                    ret[1] += &details;
                }
                return (ret, Directive::NO_FILE_COMP);
            }
        }
        (Vec::new(), Directive::NO_FILE_COMP)
    })
}
