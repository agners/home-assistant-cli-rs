use super::interface_completions;
use crate::cmd::helpers::debug_args;
use crate::cobra::{Args, Command};
use crate::{client, output};

pub fn command() -> Command {
    Command::new("info [interface]")
        .aliases(&["in", "inf"])
        .short("Shows information about the host network")
        .long(
            "
Shows information about the host network and interfaces or only from a specific interface.
",
        )
        .example(
            "
  ha network info
  ha network info eth0
",
        )
        .valid_args(interface_completions())
        .args(Args::Maximum(1))
        .run(|ctx| {
            debug_args(ctx);

            // The interface name goes into the URL as is (no path parameter).
            let command = if ctx.args.is_empty() {
                "info".to_string()
            } else {
                format!("interface/{}/info", ctx.arg(0))
            };

            output::show(client::generic_json_get("network", &command))
        })
}
