use serde_json::Value;

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{self, Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options on backup manager")
        .long(
            "
This command allows you to set configuration options for backup manager.",
        )
        .example(
            "
  ha backups options --days-until-stale 60",
        )
        .flag(Flag::int("days-until-stale", 30, "Days until backup considered stale").completion(comp::no_file()))
        // backwards compatibility
        .normalize_flags(cobra::normalize_underscores)
        .run(Call::post("backups", "options").run_with(|ctx, options| {
            let days_until_stale = ctx.int("days-until-stale");
            if days_until_stale != 0 && ctx.changed("days-until-stale") {
                options.insert("days_until_stale".to_string(), Value::from(days_until_stale));
            }
        }))
}
