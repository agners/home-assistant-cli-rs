use serde_json::Value;

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("freeze")
        .aliases(&["frz"])
        .short("Freeze supervisor for external backup")
        .long(
            "
This command tells Supervisor to prepare Home Assistant and apps for a backup
or snapshot taken by external software. Caller should call thaw when done.",
        )
        .example(
            "
  ha backups freeze --timeout 300",
        )
        .flag(Flag::int("timeout", 0, "Seconds before freeze times out and thaw begins").completion(comp::no_file()))
        .run(Call::post("backups", "freeze").run_with(|ctx, options| {
            let timeout = ctx.int("timeout");
            if timeout != 0 && ctx.changed("timeout") {
                options.insert("timeout".to_string(), Value::from(timeout));
            }
        }))
}
