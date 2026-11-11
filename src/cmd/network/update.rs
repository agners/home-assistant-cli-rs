use serde_json::Value;

use super::{
    interface_completions, ip_addr_gen_mode_completions, ip_method_completions, ip6_privacy_completions,
    mdns_completions,
};
use crate::client::Options;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, Ctx, Flag, comp};

pub fn command() -> Command {
    Command::new("update [interface]")
        .aliases(&["set", "up"])
        .short("Update settings of a network interface")
        .long(
            "
Update network interface settings of a specific adapter.
",
        )
        .example(
            "
  ha network update eth0 --ipv4-method auto --ipv6-method disabled
",
        )
        .valid_args(interface_completions())
        .args(Args::Exact(1))
        .flag(
            Flag::string_array(
                "ipv4-address",
                "IPv4 address for the interface in CIDR notation (e.g. 192.168.1.5/24)",
            )
            .completion(comp::no_file()),
        )
        .flag(Flag::string("ipv4-gateway", "", "The IPv4 gateway the interface should use").completion(comp::no_file()))
        .flag(Flag::string("ipv4-method", "", "Method on IPv4: static|auto|disabled").completion(ip_method_completions()))
        // No completion: Go registers it for the non-existing flag
        // "ipv4-nameservers", so the default (file) completion applies.
        .flag(Flag::string_array(
            "ipv4-nameserver",
            "IPv4 address of upstream DNS servers. Use multiple times for multiple servers.",
        ))
        .flag(
            Flag::int(
                "ipv4-route-metric",
                -1,
                "IPv4 route metric. Lower value has higher priority.",
            )
            .completion(comp::no_file()),
        )
        .flag(
            Flag::string_array(
                "ipv6-address",
                "IPv6 address for the interface in CIDR notation (e.g. 2001:db8:85a3::8a2e:370:7334/64)",
            )
            .completion(comp::no_file()),
        )
        .flag(Flag::string("ipv6-gateway", "", "The IPv6 gateway the interface should use").completion(comp::no_file()))
        .flag(Flag::string("ipv6-method", "", "Method on IPv6: static|auto|disabled").completion(ip_method_completions()))
        .flag(
            Flag::string(
                "ipv6-addr-gen-mode",
                "",
                "IPv6 address generation mode: eui64|stable-privacy|default-or-eui64|default",
            )
            .completion(ip_addr_gen_mode_completions()),
        )
        .flag(
            Flag::string(
                "ipv6-privacy",
                "",
                "IPv6 privacy extensions: disabled|enabled-prefer-public|enabled|default",
            )
            .completion(ip6_privacy_completions()),
        )
        // No completion: Go registers it for the non-existing flag
        // "ipv6-nameservers".
        .flag(Flag::string_array(
            "ipv6-nameserver",
            "IPv6 address for upstream DNS servers. Use multiple times for multiple servers.",
        ))
        .flag(
            Flag::int(
                "ipv6-route-metric",
                -1,
                "IPv6 route metric. Lower value has higher priority. The kernel accepts zero (0) but coerces it to 1024 (user default).",
            )
            .completion(comp::no_file()),
        )
        .flag(
            Flag::string("wifi-mode", "", "Wifi mode: infrastructure, adhoc, mesh or ap")
                .completion(comp::fixed(&["infrastructure", "adhoc", "mesh", "ap"])),
        )
        .flag(Flag::string("wifi-ssid", "", "SSID for wifi connection").completion(comp::no_file()))
        .flag(
            Flag::string("wifi-auth", "", "Used authentication: open, wep, wpa-psk")
                .completion(comp::fixed(&["open", "wep", "wpa-psk"])),
        )
        .flag(Flag::string("wifi-psk", "", "Shared authentication key for wep or wpa").completion(comp::no_file()))
        .flag(
            Flag::bool("disabled", false, "Disable interface")
                .short('e')
                .completion(comp::bools()),
        )
        .flag(Flag::string("mdns", "", "mDNS mode: default|off|resolve|announce").completion(mdns_completions()))
        .flag(Flag::string("llmnr", "", "LLMNR mode: default|off|resolve|announce").completion(mdns_completions()))
        .run(
            // The body is always sent: it holds at least "enabled".
            Call::post("network", "interface/{interface}/update")
                .arg_param("interface", 0)
                .always_body()
                .run_with(|ctx, options| {
                    // IP configs
                    helper_ip_config("ipv4", ctx, options);
                    helper_ip_config("ipv6", ctx, options);

                    // Wifi
                    helper_wifi_config(ctx, options);

                    // mDNS / LLMNR
                    helper_mdns_config(ctx, options);

                    options.insert("enabled".into(), Value::Bool(!ctx.bool("disabled")));
                }),
        )
}

/// Go: `NetworkArg`
enum Kind {
    String,
    Array,
    Int,
}

/// Go: `parseNetworkArgs`. `(flag, field, kind)`; flags that do not exist on
/// the command (e.g. `ipv4-addr-gen-mode`) are skipped, like the lookup error
/// in Go.
fn parse_network_args(ctx: &Ctx, args: &[(String, &str, Kind)]) -> Options {
    let mut config = Options::new();
    for (arg, field, kind) in args {
        if !ctx.changed(arg) {
            continue;
        }
        let val = match kind {
            Kind::Int => Value::from(ctx.int(arg)),
            Kind::Array => {
                let val = ctx.string_array(arg);
                if val.is_empty() {
                    continue;
                }
                Value::from(val)
            }
            Kind::String => {
                let val = ctx.string(arg);
                if val.is_empty() {
                    continue;
                }
                Value::String(val)
            }
        };
        config.insert(field.to_string(), val);
    }
    config
}

/// Go: `helperIpConfig`
pub(super) fn helper_ip_config(version: &str, ctx: &Ctx, options: &mut Options) {
    let args = [
        (format!("{version}-gateway"), "gateway", Kind::String),
        (format!("{version}-method"), "method", Kind::String),
        (format!("{version}-addr-gen-mode"), "addr_gen_mode", Kind::String),
        (format!("{version}-privacy"), "ip6_privacy", Kind::String),
        (format!("{version}-address"), "address", Kind::Array),
        (format!("{version}-nameserver"), "nameservers", Kind::Array),
        (format!("{version}-route-metric"), "route_metric", Kind::Int),
    ];
    let ip_config = parse_network_args(ctx, &args);
    if !ip_config.is_empty() {
        options.insert(version.to_string(), Value::Object(ip_config));
    }
}

/// Go: `helperWifiConfig`
fn helper_wifi_config(ctx: &Ctx, options: &mut Options) {
    let args = [
        ("wifi-mode".to_string(), "mode", Kind::String),
        ("wifi-ssid".to_string(), "ssid", Kind::String),
        ("wifi-auth".to_string(), "auth", Kind::String),
        ("wifi-psk".to_string(), "psk", Kind::String),
    ];
    let wifi_config = parse_network_args(ctx, &args);
    if !wifi_config.is_empty() {
        options.insert("wifi".to_string(), Value::Object(wifi_config));
    }
}

/// Go: `helperMdnsConfig`
pub(super) fn helper_mdns_config(ctx: &Ctx, options: &mut Options) {
    for key in ["mdns", "llmnr"] {
        let value = ctx.string(key);
        if !value.is_empty() {
            options.insert(key.to_string(), Value::String(value));
        }
    }
}
