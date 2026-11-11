use super::update::{helper_ip_config, helper_mdns_config};
use super::{
    interface_completions, ip_addr_gen_mode_completions, ip_method_completions, ip6_privacy_completions,
    mdns_completions,
};
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, Flag, comp};

pub fn command() -> Command {
    Command::new("vlan [interface] [id]")
        .short("Create a new VLAN on an ethernet interface")
        .long(
            "
Create a new VLAN on an ethernet interface. It allows setting an initial IP config.
This function works only on an ethernet interface!
",
        )
        .example(
            "
  ha network vlan eth0 10 --ipv4-method auto --ipv6-method disabled
",
        )
        .valid_args(interface_completions())
        .args(Args::Exact(2))
        .flag(
            Flag::string_array(
                "ipv4-address",
                "IPv4 address for the interface in CIDR notation (e.g. 192.168.1.5/24)",
            )
            .completion(comp::no_file()),
        )
        .flag(Flag::string("ipv4-gateway", "", "The IPv4 gateway the interface should use").completion(comp::no_file()))
        .flag(Flag::string("ipv4-method", "", "Method on IPv4: static|auto|disabled").completion(ip_method_completions()))
        .flag(
            Flag::string_array(
                "ipv4-nameserver",
                "IPv4 address of upstream DNS servers. Use multiple times for multiple servers.",
            )
            .completion(comp::no_file()),
        )
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
        .flag(
            Flag::string_array(
                "ipv6-nameserver",
                "IPv6 address for upstream DNS servers. Use multiple times for multiple servers.",
            )
            .completion(comp::no_file()),
        )
        .flag(
            Flag::int(
                "ipv6-route-metric",
                -1,
                "IPv6 route metric. Lower value has higher priority. The kernel accepts zero (0) but coerces it to 1024 (user default).",
            )
            .completion(comp::no_file()),
        )
        .flag(Flag::string("mdns", "", "mDNS mode: default|off|resolve|announce").completion(mdns_completions()))
        .flag(Flag::string("llmnr", "", "LLMNR mode: default|off|resolve|announce").completion(mdns_completions()))
        .run(
            // The body is only sent when at least one option is set.
            Call::post("network", "interface/{interface}/vlan/{vlan}")
                .arg_param("interface", 0)
                .arg_param("vlan", 1)
                .run_with(|ctx, options| {
                    // IP configs
                    helper_ip_config("ipv4", ctx, options);
                    helper_ip_config("ipv6", ctx, options);

                    // mDNS / LLMNR
                    helper_mdns_config(ctx, options);
                }),
        )
}
