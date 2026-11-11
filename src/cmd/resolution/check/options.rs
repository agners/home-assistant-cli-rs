use super::check_completions;
use crate::cmd::helpers::{Call, bool_if_changed};
use crate::cobra::{Args, Command, Flag, comp};

pub fn command() -> Command {
    Command::new("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Options apply to checks managed by the Resolution center")
        .long(
            "
This command allows to apply options to an specific check managed by the system.",
        )
        .example(
            "
  ha resolution check options [slug]",
        )
        .valid_args(check_completions())
        .args(Args::Exact(1))
        .flag(
            Flag::bool("enabled", true, "Enable/Disable check on the system")
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(
            Call::post("resolution", "check/{check}/options")
                .arg_param("check", 0)
                .run_with(|ctx, options| bool_if_changed(ctx, options, "enabled", "enabled")),
        )
}
