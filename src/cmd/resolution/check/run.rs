use super::check_completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("run")
        .aliases(&["execute", "ru"])
        .short("Run a specific check at the Resolution center")
        .long(
            "
This command executes an backend check immediately on the system.",
        )
        .example(
            "
  ha resolution check run [slug]",
        )
        .valid_args(check_completions())
        .args(Args::Exact(1))
        .run(
            Call::post("resolution", "check/{check}/run")
                .arg_param("check", 0)
                .spinner()
                .run(),
        )
}
