use crate::cmd::completions;
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};
use crate::{client, output};

pub fn command() -> Command {
    Command::new("info [slug]")
        .aliases(&["in", "info"])
        .short("Show information about available Home Assistant apps")
        .long(
            "
This command can provide information on all available apps or, if a slug
is provided, information about a specific app.
",
        )
        .example(
            "
  ha apps info
  ha apps info core_ssh
",
        )
        .valid_args(completions::apps())
        .args(Args::Maximum(1))
        .run(|ctx| {
            debug_args(ctx);

            let url = match client::url_helper("addons", "{slug}/info") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let slug = if ctx.args.is_empty() { "self" } else { ctx.arg(0) };

            let resp = client::json_request().path_param("slug", slug).get(&url);
            output::show(client::generic_json_error_handling(resp))
        })
}
