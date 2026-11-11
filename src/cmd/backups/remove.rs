use serde_json::Value;

use super::{backups_completions, string_array};
use crate::client::{self, Options};
use crate::cmd::completions;
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command, Flag};
use crate::{debug, gofmt, output};

pub fn command() -> Command {
    Command::new("remove [slug]")
        .aliases(&["delete", "del", "rem", "rm"])
        .short("Deletes a backup from disk")
        .long(
            "
Backups can take quite a bit of diskspace, this command allows you to
clean backups from disk.",
        )
        .example(
            "
  ha backups remove c1a07617",
        )
        .valid_args(backups_completions())
        .args(Args::Exact(1))
        .flag(
            Flag::string_array(
                "location",
                "location(s) to remove backup from (instead of all), use multiple times for multiple locations.",
            )
            .short('l')
            .no_opt_default(".local")
            .completion(completions::backups_locations()),
        )
        .run(|ctx| {
            debug_args(ctx);

            let url = match client::url_helper("backups", "{slug}") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let mut request = client::json_request().path_param("slug", ctx.arg(0));
            let mut options = Options::new();

            let location = string_array(ctx, "location");
            debug!("location", "location" => gofmt::list(&location));
            if !location.is_empty() && ctx.changed("location") {
                options.insert("location".to_string(), Value::from(location));
            }

            if !options.is_empty() {
                debug!("Request body", "options" => gofmt::json_marshal(&Value::Object(options.clone())));
                request = request.body(options);
            }

            let result = request.delete(&url);
            output::show(client::generic_json_error_handling(result))
        })
}
