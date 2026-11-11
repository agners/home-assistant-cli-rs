use crate::cmd::helpers::{Call, bool_if_changed, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options for the Security backend")
        .long(
            "
This command allows you to set configuration options for the internally
Home Assistant Security backend.
",
        )
        .example(
            "
  ha security options --force-security=True
",
        )
        .flag(
            Flag::bool("pwned", true, "Enable/Disable pwned check the backend")
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .flag(
            Flag::bool("force-security", false, "Enable/Disable force-security on the backend")
                .no_opt_default("false")
                .completion(comp::bools()),
        )
        .run(Call::post("security", "options").run_with(|ctx, options| {
            bool_if_changed(ctx, options, "pwned", "pwned");
            bool_if_changed(ctx, options, "force-security", "force_security");
        }))
}
