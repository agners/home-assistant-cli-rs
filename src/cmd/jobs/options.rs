use serde_json::Value;

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options for the Job Manager backend")
        .long(
            "
This command allows you to set configuration options for the internally
Home Assistant Job Manager.
",
        )
        .example(
            "
  ha jobs options --ignore-conditions healthy
",
        )
        .flag(
            Flag::string_array(
                "ignore-conditions",
                "Conditions to ignore on Job Manager. Use multiple times for ignored conditions.",
            )
            .short('i')
            .completion(comp::no_file()),
        )
        .run(Call::post("jobs", "options").run_with(|ctx, options| {
            let mut conditions = ctx.string_array("ignore-conditions");
            // pflag's GetStringArray re-parses the flag's "[a,b]" text form,
            // which turns a single empty value into an empty list.
            if conditions == [""] {
                conditions.clear();
            }
            if !conditions.is_empty() {
                options.insert("ignore_conditions".to_string(), Value::from(conditions));
            }
        }))
}
