use super::suggestion_completions;
use crate::client;
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};
use crate::output;

pub fn command() -> Command {
    Command::new("dismiss")
        .aliases(&["disable", "remove"])
        .short("Suggestion dismiss reported by Resolution center")
        .long(
            "
This command allows dismissing a suggestion reported by the system.",
        )
        .example(
            "
  ha resolution suggestion dismiss [id]",
        )
        .valid_args(suggestion_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);

            let url = match client::url_helper("resolution", "suggestion/{suggestion}") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let resp = client::json_request().path_param("suggestion", ctx.arg(0)).delete(&url);
            output::show(client::generic_json_error_handling(resp))
        })
}
