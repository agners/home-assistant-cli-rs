use serde_json::Value;

use super::{logs_boots, logs_identifiers};
use crate::cmd::completions;
use crate::cmd::helpers::{debug_args, leaf};
use crate::cmd::logs::{add_logs_flags, process_logs_flags, send_and_stream};
use crate::cobra::{Command, Completer, Directive, Flag, comp};

pub fn command() -> Command {
    add_logs_flags(
        leaf("logs")
            .aliases(&["log", "lg"])
            .short("View the log output of the host systemd journal logs")
            .long(
                "
Allows you to look at the systemd journal on the host to see logs
across services and boots.
",
            )
            .example(
                "
  ha host logs
",
            ),
    )
    .flag(
        Flag::string("identifier", "", "Show entries with the specified syslog identifier")
            .short('t')
            .completion(identifiers_completions()),
    )
    .run(|ctx| {
        debug_args(ctx);
        let request = process_logs_flags("host", ctx).map(|mut request| {
            let identifier = ctx.string("identifier");
            if !identifier.is_empty() {
                if request.url.ends_with("/follow") {
                    // We can safely do this because "/follow" will be always at the end
                    request.url = request.url.replacen("/follow", "/identifiers/{identifier}/follow", 1);
                } else {
                    request.url.push_str("/identifiers/{identifier}");
                }
            }
            request.path_param("identifier", &identifier)
        });
        send_and_stream(request)
    })
    .subcommand(logs_boots::command())
    .subcommand(logs_identifiers::command())
}

/// Go: `hostLogsIdentifiersCompletions`
fn identifiers_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Some(data) = completions::api_data("host", "logs/identifiers") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for id in data.get("identifiers").and_then(Value::as_array).into_iter().flatten() {
            if let Some(identifier) = id.as_str() {
                ret.push(identifier.to_string());
            }
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
