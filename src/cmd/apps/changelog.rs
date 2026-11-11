use std::io::Read;

use crate::cmd::completions;
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};
use crate::{client, log, output};

pub fn command() -> Command {
    Command::new("changelog [slug]")
        .aliases(&["cl", "ch"])
        .short("Show changelog of a Home Assistant app")
        .long(
            "
This command shows the changelog of an app. It gives you what has been
changed in the latest version and tell you about possible breaking changes.",
        )
        .example(
            "
ha apps changelog core_ssh
ha apps changelog core_mosquitto",
        )
        .valid_args(completions::apps())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);

            let url = match client::url_helper("addons", "{slug}/changelog") {
                Ok(url) => url,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };

            let result = client::request()
                .path_param("slug", ctx.arg(0))
                .header("Accept", "text/plain")
                .get(&url)
                .and_then(|mut resp| {
                    // The plain request does not buffer the body; read it
                    // like resty does before handing out the response.
                    let mut body = Vec::new();
                    resp.reader()
                        .read_to_end(&mut body)
                        .map_err(|e| client::Error::new(e.to_string()))?;
                    Ok((resp.status_code(), body))
                })
                .and_then(|(status, body)| {
                    // returns 200 OK or 400, everything else is wrong
                    if status != 200 && status != 400 {
                        log::log(
                            log::ERROR,
                            "unexpected server response",
                            &[("status", status.to_string())],
                        );
                        return Err(client::Error::new("unexpected server response"));
                    }
                    Ok(body)
                });

            match result {
                Ok(body) => {
                    output::print_bytes(&body);
                    output::print_bytes(b"\n");
                    true
                }
                Err(err) => {
                    output::print_error(&err);
                    false
                }
            }
        })
}
