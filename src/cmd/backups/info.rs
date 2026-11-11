use super::backups_completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("info [slug]")
        .aliases(&["in", "inf"])
        .short("Provides information about the current available backups")
        .long(
            "
When a Home Assistant backup is created, it will be available for restore.
This command gives you information about a specific backup.",
        )
        .example(
            "
  ha backups info c1a07617",
        )
        .valid_args(backups_completions())
        .args(Args::Exact(1))
        .run(Call::get("backups", "{slug}/info").arg_param("slug", 0).run())
}
