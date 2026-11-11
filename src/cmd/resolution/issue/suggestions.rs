use super::issue_completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("suggestions")
        .aliases(&["su", "solutions"])
        .short("Suggestions which resolve an issue")
        .long(
            "
This command returns suggestions which resolve an issue when applied.",
        )
        .example(
            "
  ha resolution issue suggestions [id]",
        )
        .valid_args(issue_completions())
        .args(Args::Exact(1))
        .run(
            Call::get("resolution", "issue/{issue}/suggestions")
                .arg_param("issue", 0)
                .run(),
        )
}
