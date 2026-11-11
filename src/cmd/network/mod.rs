//! `ha network` — host network information and configuration
//! (Go: `cmd/network*.go`).

mod info;
mod reload;
mod scan;
mod update;
mod vlan;

use serde_json::Value;

use crate::cmd::completions;
use crate::cobra::{Command, Completer, Directive, comp};

pub fn command() -> Command {
    Command::new("network")
        .aliases(&["net"])
        .short("Network specific for updating, info and configuration imports")
        .long(
            "
The network command provides command line tools to control the host network that
Home Assistant is running on. It allows you to do things like change the
system network IP address, set connection options or join a Wi-Fi network.",
        )
        .example(
            "
  ha network info
  ha network interface options",
        )
        .subcommand(info::command())
        .subcommand(reload::command())
        .subcommand(scan::command())
        .subcommand(update::command())
        .subcommand(vlan::command())
}

/// Go: `ipMethodCompletions`
fn ip_method_completions() -> Completer {
    comp::fixed(&["static", "auto", "disabled"])
}

/// Go: `ipAddrGenModeCompletions`
fn ip_addr_gen_mode_completions() -> Completer {
    comp::fixed(&["eui64", "stable-privacy", "default-or-eui64", "default"])
}

/// Go: `ip6PrivacyCompletions`
fn ip6_privacy_completions() -> Completer {
    comp::fixed(&["disabled", "enabled-prefer-public", "enabled", "default"])
}

/// Go: `mdnsCompletions`
fn mdns_completions() -> Completer {
    comp::fixed(&["default", "off", "resolve", "announce"])
}

/// Go: `networkInterfaceCompletions` — interface names, filtered for `scan`
/// (enabled wireless interfaces) and `vlan` (ethernet interfaces).
fn interface_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = completions::api_data("network", "info") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for iface in data.get("interfaces").and_then(Value::as_array).into_iter().flatten() {
            let Some(m) = iface.as_object() else { continue };
            let kind = m.get("type").and_then(Value::as_str);
            let wanted = match ctx.name() {
                "scan" => m.get("enabled").and_then(Value::as_bool) == Some(true) && kind == Some("wireless"),
                "vlan" => kind == Some("ethernet"),
                _ => true,
            };
            if !wanted {
                continue;
            }
            match m.get("interface").and_then(Value::as_str) {
                Some(s) if !s.is_empty() => ret.push(s.to_string()),
                _ => {}
            }
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
