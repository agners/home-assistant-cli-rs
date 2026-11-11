use serde_json::Value;

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("profile")
        .aliases(&["pro"])
        .short("Set the Home Assistant Audio profile for a card")
        .long(
            "
This command allows you to set the audio profile on a audio card.",
        )
        .example(
            "
\tha audio profile --card \"...\" --name \"...\"
",
        )
        .flag(
            Flag::string("name", "", "Name of the profile")
                .required()
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("card", "", "The card to set the profile for")
                .required()
                .completion(comp::no_file()),
        )
        .run(Call::post("audio", "profile").run_with(|ctx, options| {
            let name = ctx.string("name");
            if !name.is_empty() && ctx.changed("name") {
                options.insert("name".to_string(), Value::String(name.clone()));
            }
            // Go checks `name` (not `card`) for being non-empty here as well.
            if !name.is_empty() && ctx.changed("card") {
                options.insert("card".to_string(), Value::String(ctx.string("card")));
            }
        }))
}
