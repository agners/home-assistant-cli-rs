use crate::cmd::completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("start [slug]")
        .aliases(&["run", "st"])
        .short("Manually start a stopped Home Assistant app")
        .long(
            "
This command allows you to manually start a stopped Home Assistant app
",
        )
        .example(
            "
  ha apps start core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .run(Call::post("addons", "{slug}/start").arg_param("slug", 0).run())
}
