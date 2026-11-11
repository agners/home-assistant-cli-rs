use super::suggestion_completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("apply")
        .aliases(&["enable", "run"])
        .short("Suggestion apply reported by Resolution center")
        .long(
            "
This command allow to apply an suggestion reported by the System.",
        )
        .example(
            "
  ha resolution suggestion apply [id]",
        )
        .valid_args(suggestion_completions())
        .args(Args::Exact(1))
        .run(
            Call::post("resolution", "suggestion/{suggestion}")
                .arg_param("suggestion", 0)
                .run(),
        )
}
