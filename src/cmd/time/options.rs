use serde_json::Value;

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::{Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Set Home Assistant OS time options")
        .long(
            "
This command allows you to set Home Assistant OS NTP servers and fallback NTP
servers.",
        )
        .example(
            "
  ha time options --servers pool.ntp.org --servers time.cloudflare.com
  ha time options --fallback-servers time.google.com
  ha time options --clear-fallback-servers",
        )
        .flag(
            Flag::string_array(
                "servers",
                "NTP servers to use. Use multiple times for multiple servers.",
            )
            .short('s')
            .completion(comp::no_file()),
        )
        .flag(
            Flag::string_array(
                "fallback-servers",
                "Fallback NTP servers to use. Use multiple times for multiple servers.",
            )
            .short('f')
            .completion(comp::no_file()),
        )
        .flag(Flag::bool("clear-servers", false, "Clear configured NTP servers").completion(comp::bools()))
        .flag(
            Flag::bool("clear-fallback-servers", false, "Clear configured fallback NTP servers")
                .completion(comp::bools()),
        )
        .mutually_exclusive(&["servers", "clear-servers"])
        .mutually_exclusive(&["fallback-servers", "clear-fallback-servers"])
        .run(Call::post("time", "options").run_with(|ctx, options| {
            if ctx.changed("servers") {
                options.insert("servers".into(), Value::from(ctx.string_array("servers")));
            }
            if ctx.bool("clear-servers") {
                options.insert("servers".into(), Value::Array(Vec::new()));
            }
            if ctx.changed("fallback-servers") {
                options.insert(
                    "fallback_servers".into(),
                    Value::from(ctx.string_array("fallback-servers")),
                );
            }
            if ctx.bool("clear-fallback-servers") {
                options.insert("fallback_servers".into(), Value::Array(Vec::new()));
            }
        }))
}
