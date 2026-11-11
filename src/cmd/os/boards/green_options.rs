use crate::cmd::helpers::{Call, bool_if_changed, leaf};
use crate::cobra::{Command, Flag, comp};

fn led_flag(name: &str, usage: &str) -> Flag {
    Flag::bool(name, true, usage)
        .no_opt_default("true")
        .completion(comp::bools())
}

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Change settings of the current Green board")
        .long(
            "
This command allows you to change settings of the Green board that Home
Assistant is running on.",
        )
        .example(
            "
  ha os boards green options --activity-led=false",
        )
        .flag(led_flag("activity-led", "Enable/disable the green activity LED"))
        .flag(led_flag("power-led", "Enable/disable the white power LED"))
        .flag(led_flag(
            "system-health-led",
            "Enable/disable the yellow system health LED",
        ))
        .run(Call::post("os", "boards/green").run_with(|ctx, options| {
            for value in ["activity-led", "power-led", "system-health-led"] {
                bool_if_changed(ctx, options, value, &value.replace('-', "_"));
            }
        }))
}
