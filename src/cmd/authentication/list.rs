use super::debug_args;
use crate::cmd::helpers::leaf;
use crate::cobra::Command;
use crate::{client, output};

pub fn command() -> Command {
    leaf("list")
        .aliases(&["ls"])
        .short("List all Home Assistant users.")
        .long(
            "
This command allows you to list all Home Assistant users on the system.
Please note, this command is limited due to security reasons, and will
only work on some locations. For example, the Operating System CLI.
",
        )
        .example(
            "
  ha authentication list
",
        )
        .run(|ctx| {
            debug_args(ctx);

            match client::generic_json_get("auth", "list") {
                Ok(mut resp) => output::show_json_response(&mut resp),
                Err(err) => {
                    eprintln!(
                        "this command is limited due to security reasons, and will only work on some locations. For example, the Operating System terminal."
                    );
                    output::print_error(&err);
                    false
                }
            }
        })
}
