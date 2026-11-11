//! Shared `logs` flags and request construction (Go: `addLogsFlags` and
//! `processLogsFlags` in `cmd/host.go`).

use crate::client::{self, Request};
use crate::cmd::completions;
use crate::cmd::helpers::debug_args;
use crate::cobra::{Command, Ctx, Flag, comp};
use crate::{debug, output};

/// `addLogsFlags(cmd)`: `--follow/-f`, `--lines/-n`, `--boot/-b`,
/// `--verbose/-v`.
pub fn add_logs_flags(cmd: Command) -> Command {
    cmd.flag(
        Flag::bool("follow", false, "Continuously print new log entries")
            .short('f')
            .completion(comp::bools()),
    )
    .flag(
        Flag::uint32("lines", 0, "Number of log entries to show")
            .short('n')
            .completion(comp::no_file()),
    )
    .flag(
        Flag::string("boot", "", "Logs of particular boot ID")
            .short('b')
            .completion(completions::host_boot()),
    )
    .flag(
        Flag::bool("verbose", false, "Return logs in verbose format")
            .short('v')
            .completion(comp::bools()),
    )
}

/// `processLogsFlags(section, cmd)`: build the (not yet sent) logs request.
/// `request.url` and `request.method` can still be adjusted before
/// `request.send()`.
pub fn process_logs_flags(section: &str, ctx: &Ctx) -> client::Result<Request> {
    let mut command = String::from("logs");

    let boot = ctx.string("boot");
    if !boot.is_empty() {
        command.push_str("/boots/{boot}");
    }

    if ctx.bool("follow") {
        command.push_str("/follow");
    }

    let url = client::url_helper(section, &command)?;

    let accept = if ctx.bool("verbose") {
        "text/x-log"
    } else {
        "text/plain"
    };

    /* Disable timeouts to allow following forever */
    let mut request = client::request_timeout(client::NO_TIMEOUT)
        .header("Accept", accept)
        .do_not_parse();

    let lines = ctx.uint32("lines");
    if lines > 0 {
        let range_header = format!("entries=:{}:", -(i64::from(lines) - 1));
        debug!("Range header", "value" => range_header);
        request = request.header("Range", &range_header);
    }

    request = request.path_param("boot", &boot);
    request.url = url;
    Ok(request)
}

/// Send a logs request and stream the text response to stdout.
pub fn send_and_stream(request: client::Result<Request>) -> bool {
    match request.and_then(|r| r.send()) {
        Ok(mut resp) => output::stream_text_response(&mut resp),
        Err(err) => {
            output::print_error(&err);
            false
        }
    }
}

/// `Run` of the plain `<section> logs` commands.
pub fn run_logs(section: &str) -> impl Fn(&Ctx) -> bool {
    let section = section.to_string();
    move |ctx| {
        debug_args(ctx);
        send_and_stream(process_logs_flags(&section, ctx))
    }
}
