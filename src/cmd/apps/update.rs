use std::time::Duration;

use crate::client::{self, Options};
use crate::cmd::completions;
use crate::cmd::helpers::{bool_if_changed, debug_args};
use crate::cobra::{Args, Command, Ctx, Flag, comp};
use crate::{output, spinner};

/// The `--backup` flag of `apps update` and `store apps update`.
pub fn backup_flag() -> Flag {
    Flag::bool("backup", false, "Create partial backup before update").completion(comp::bools())
}

/// `Run` shared by `apps update` and `store apps update`: POST
/// `<section>/<command>` (with a `{slug}` path parameter from arg 0) and
/// `{"backup": ...}` if `--backup` was given.
///
/// Quirk kept from Go: the spinner is started and stopped around the
/// *creation* of the request, not around the request itself.
pub fn run_update(section: &'static str, command: &'static str, timeout: Duration) -> impl Fn(&Ctx) -> bool {
    move |ctx| {
        debug_args(ctx);

        let url = match client::url_helper(section, command) {
            Ok(url) => url,
            Err(err) => {
                output::print_error(&err);
                return false;
            }
        };

        spinner::start();
        let mut request = client::json_request_timeout(timeout);
        spinner::stop();

        request = request.path_param("slug", ctx.arg(0));

        let mut options = Options::new();
        bool_if_changed(ctx, &mut options, "backup", "backup");
        if !options.is_empty() {
            request = request.body(options);
        }

        let resp = request.post(&url);
        output::show(client::generic_json_error_handling(resp))
    }
}

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
  ha apps update core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .flag(backup_flag())
        .run(run_update(
            "addons",
            "{slug}/update",
            client::CONTAINER_DOWNLOAD_TIMEOUT,
        ))
}
