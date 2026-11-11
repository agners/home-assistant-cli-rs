use crate::client::{self, Options};
use crate::cmd::helpers::{Call, debug_args, int_if_changed, leaf, string_if_changed};
use crate::cobra::{Command, Directive, Flag, comp};

const MAX_SWAPPINESS: u32 = 200;

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Change HAOS swap settings")
        .long(
            "
This command allows you to override how the Home Assistant OS uses swap.",
        )
        .example(
            "
  ha os config swap options --swap-size=2G --swappiness=10",
        )
        .flag(
            Flag::string("swap-size", "", "Swap size in bytes with optional units (K/M/G)").completion(comp::no_file()),
        )
        .flag(
            Flag::int(
                "swappiness",
                1,
                &format!("Kernel swappiness value (0-{MAX_SWAPPINESS})"),
            )
            .completion(comp::from_fn(|_, _| {
                let v = (0..=MAX_SWAPPINESS).map(|i| i.to_string()).collect();
                (v, Directive::NO_FILE_COMP)
            })),
        )
        .run(|ctx| {
            debug_args(ctx);

            let mut options = Options::new();
            string_if_changed(ctx, &mut options, "swap-size", "swap_size");
            int_if_changed(ctx, &mut options, "swappiness", "swappiness");
            let changed = !options.is_empty();

            let success = Call::post("os", "config/swap").execute(ctx, options);
            if success && changed && !client::raw_json() {
                println!("\nSwap settings saved. Reboot the device using `ha host reboot` to apply the changes.");
            }
            success
        })
}
