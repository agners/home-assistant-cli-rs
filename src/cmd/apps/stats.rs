use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("stats [slug]")
        .aliases(&["status", "stat"])
        .short("Provides system usage stats of a Home Assistant app")
        .long(
            "
Provides insight into the system usage stats of an app. It shows you
how much CPU, memory, disk & network resources it uses.
",
        )
        .example(
            "
  ha apps stats core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .run(Call::get("addons", "{slug}/stats").arg_param("slug", 0).run())
}
