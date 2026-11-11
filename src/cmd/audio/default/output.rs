use serde_json::Value;

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("output")
        .aliases(&["out"])
        .short("Set the default Home Assistant Audio output channel")
        .long(
            "
This command allows you to set the default output channel of the
Home Assistant Audio on your Home Assistant system.",
        )
        .example(
            "
\tha audio default output --name \"...\"
",
        )
        .flag(
            Flag::string("name", "", "The name of the audio device")
                .required()
                .completion(comp::no_file()),
        )
        .run(Call::post("audio", "default/output").run_with(|ctx, options| {
            let name = ctx.string("name");
            if !name.is_empty() && ctx.changed("name") {
                options.insert("name".to_string(), Value::String(name));
            }
        }))
}
