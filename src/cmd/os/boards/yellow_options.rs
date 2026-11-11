use crate::client::{self, Options};
use crate::cmd::helpers::{Call, bool_if_changed, debug_args, leaf};
use crate::cobra::{Command, Flag, comp};

fn led_flag(name: &str, usage: &str) -> Flag {
    Flag::bool(name, true, usage)
        .no_opt_default("true")
        .completion(comp::bools())
}

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Change settings of the current Yellow board")
        .long(
            "
This command allows you to change settings of the Yellow board that Home
Assistant is running on. A host reboot is required for changes to take effect.",
        )
        .example(
            "
  ha os boards yellow options --heartbeat-led=false",
        )
        .flag(led_flag("disk-led", "Enable/disable the disk LED"))
        .flag(led_flag("heartbeat-led", "Enable/disable the heartbeat LED"))
        .flag(led_flag("power-led", "Enable/disable the power LED"))
        .run(|ctx| {
            debug_args(ctx);

            let mut options = Options::new();
            for value in ["disk-led", "heartbeat-led", "power-led"] {
                bool_if_changed(ctx, &mut options, value, &value.replace('-', "_"));
            }
            let changed = !options.is_empty();

            let success = Call::post("os", "boards/yellow").execute(ctx, options);
            if success && changed && !client::raw_json() {
                println!(
                    "\nYellow board settings saved. Reboot the device using `ha host reboot` to apply the changes."
                );
            }
            success
        })
}
