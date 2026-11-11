use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};
use crate::{client, output};

pub fn command() -> Command {
    Command::new("delete [slug]")
        .aliases(&["del", "remove"])
        .short("Delete repository from Home Assistant store")
        .long(
            "
Remove a repository of apps that isn't in use from the Home Assistant store.
",
        )
        .example(
            "
ha store delete 94cfad5a
",
        )
        .valid_args(super::repositories_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);

            let url = match client::url_helper("store", "repositories/{slug}") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let resp = client::json_request().path_param("slug", ctx.arg(0)).delete(&url);
            output::show(client::generic_json_error_handling(resp))
        })
}
