use serde_json::Value;

use crate::cmd::helpers::{Call, bool_if_changed, leaf};
use crate::cobra::{self, Command, Flag, comp};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options on Home Assistant Core instance")
        .long(
            "
This command allows you to set configuration options for the Home Assistant Core
instance running on your Home Assistant system.",
        )
        .example(
            "
  ha core options --backups-exclude-database=true",
        )
        .flag(Flag::bool("boot", true, "Start Core on boot").completion(comp::bools()))
        .flag(Flag::string("image", "", "Optional image").completion(comp::no_file()))
        .flag(Flag::int("port", 8123, "Port to access Home Assistant Core").completion(comp::no_file()))
        .flag(Flag::bool("ssl", false, "Use SSL").completion(comp::bools()))
        .flag(Flag::bool("watchdog", true, "Use watchdog").completion(comp::bools()))
        .flag(Flag::string("refresh-token", "", "Refresh token").completion(comp::no_file()))
        .flag(Flag::string("audio-input", "", "Profile name for audio input").completion(comp::no_file()))
        .flag(Flag::string("audio-output", "", "Profile name for audio output").completion(comp::no_file()))
        .flag(
            Flag::bool(
                "backups-exclude-database",
                false,
                "Backups exclude Home Assistant database file by default",
            )
            .no_opt_default("false")
            .completion(comp::bools()),
        )
        .flag(
            Flag::bool(
                "duplicate-log-file",
                false,
                "Duplicate logs to file alongside Systemd Journal",
            )
            .no_opt_default("false")
            .completion(comp::bools()),
        )
        // backwards compatibility
        .normalize_flags(cobra::normalize_underscores)
        .run(Call::post("core", "options").run_with(|ctx, options| {
            for name in ["image", "refresh-token", "audio-output", "audio-input"] {
                if ctx.changed(name) {
                    let val = ctx.string(name);
                    // An explicitly empty value resets the option (JSON null).
                    let val = if val.is_empty() {
                        Value::Null
                    } else {
                        Value::String(val)
                    };
                    options.insert(name.replace('-', "_"), val);
                }
            }

            let port = ctx.int("port");
            if port != 0 && ctx.changed("port") {
                options.insert("port".to_string(), Value::from(port));
            }

            for name in [
                "boot",
                "ssl",
                "watchdog",
                "backups-exclude-database",
                "duplicate-log-file",
            ] {
                bool_if_changed(ctx, options, name, &name.replace('-', "_"));
            }
        }))
}
