use serde_json::Value;

use crate::client::{self, Options};
use crate::cmd::helpers::{Call, bool_if_changed, debug_args, leaf};
use crate::cobra::{self, Command, Flag, comp};
use crate::output;

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allows you to set options on the host docker backend")
        .long(
            "
This command allows you to set configuration options for on the host
docker backend running on your Home Assistant system.",
        )
        .example(
            "
  ha docker options --enable-ipv6=true
  ha docker options --mtu=1450",
        )
        .flag(Flag::bool("enable-ipv6", false, "Enable IPv6").completion(comp::bools()))
        .flag(Flag::int("mtu", 0, "Set Docker MTU (68-65535, 0 to reset)").completion(comp::no_file()))
        .normalize_flags(cobra::normalize_underscores)
        .run(|ctx| {
            debug_args(ctx);

            let mut options = Options::new();
            bool_if_changed(ctx, &mut options, "enable-ipv6", "enable_ipv6");

            if ctx.changed("mtu") {
                let mtu = ctx.int("mtu");
                if mtu == 0 {
                    options.insert("mtu".into(), Value::Null);
                } else if !(68..=65535).contains(&mtu) {
                    output::print_error_string("MTU value must be between 68 and 65535, or 0 to reset");
                    return false;
                } else {
                    options.insert("mtu".into(), Value::from(mtu));
                }
            }

            let changed = !options.is_empty();
            if !Call::post("docker", "options").execute(ctx, options) {
                return false;
            }
            if changed && !client::raw_json() {
                println!(
                    "\nDocker configuration saved. Reboot the device using `ha host reboot` to apply the changes."
                );
            }
            true
        })
}
