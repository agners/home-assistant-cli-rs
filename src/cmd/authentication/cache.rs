use super::debug_args;
use crate::cmd::helpers::leaf;
use crate::cobra::Command;
use crate::{client, output};

pub fn command() -> Command {
    leaf("cache")
        .aliases(&["data", "ca"])
        .short("Reset the auth cache of Home Assistant on Supervisor.")
        .long(
            "
This command allows you to reset the internal password cache of a Home Assistant auth.
",
        )
        .example(
            "
  ha authentication cache
",
        )
        .run(|ctx| {
            debug_args(ctx);

            let url = match client::url_helper("auth", "cache") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let result = client::json_request().delete(&url);
            output::show(client::generic_json_error_handling(result))
        })
}
