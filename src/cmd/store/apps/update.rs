use crate::client;
use crate::cmd::apps::update::{backup_flag, run_update};
use crate::cmd::completions;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("update [slug]")
        .aliases(&["upgrade", "up"])
        .short("Upgrades a Home Assistant app to the latest version")
        .long(
            "
This command can upgrade a Home Assistant app to its latest version.
It is currently not possible to upgrade/downgrade to a specific version.
",
        )
        .example(
            "
  ha store apps update core_ssh
",
        )
        .valid_args(completions::store_apps())
        .args(Args::Exact(1))
        .flag(backup_flag())
        // Note: unlike `apps update`, Go uses the default timeout here.
        .run(run_update("store", "addons/{slug}/update", client::DEFAULT_TIMEOUT))
}
