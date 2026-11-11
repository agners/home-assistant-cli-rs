use serde_json::Value;

use crate::cmd::completions;
use crate::cobra::{Args, Command, Completer, Directive, comp};
use crate::{client, debug, gofmt, output};

pub fn command() -> Command {
    Command::new("delete [host]")
        .aliases(&["del", "remove"])
        .short("Delete docker registry login for specific host")
        .long(
            "
Remove login for the Docker OCI registry server.
",
        )
        // Note: the stray trailing quote is in the Go source.
        .example(
            "
  ha docker registries delete my-docker.example.com\"
",
        )
        .valid_args(delete_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug!("registries delete", "args" => gofmt::list(&ctx.args));

            let url = match client::url_helper("docker", "registries/{host}") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let result = client::json_request().path_param("host", ctx.arg(0)).delete(&url);
            output::show(client::generic_json_error_handling(result))
        })
}

/// Go: `dockerRegistriesDeleteCompletions` (Go iterates the map in random
/// order; here the order is sorted).
fn delete_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = completions::api_data("docker", "registries") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        if let Some(registries) = data.get("registries").and_then(Value::as_object) {
            ret.extend(registries.keys().cloned());
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
