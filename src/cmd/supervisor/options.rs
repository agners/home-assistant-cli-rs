use serde_json::Value;

use crate::client::Options;
use crate::cmd::completions;
use crate::cmd::helpers::{Call, bool_if_changed, debug_args, leaf};
use crate::cobra::flags::parse_bool;
use crate::cobra::{Command, Completer, Ctx, Directive, Flag, comp};
use crate::{gofmt, output};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allows you to set options on the Home Assistant Supervisor")
        .long(
            "
This command allows you to set configuration options for on the Home Assistant
Supervisor running on your Home Assistant system.",
        )
        .example(
            "
  ha supervisor options --channel beta
  ha supervisor options --feature-flag supervisor_v2_api=true",
        )
        .flag(Flag::string("hostname", "", "Hostname to set").completion(comp::no_file()))
        .flag(
            Flag::string("channel", "", "Channel to track (stable|beta|dev)")
                .short('c')
                .completion(comp::fixed(&["stable", "beta", "dev"])),
        )
        .flag(
            Flag::string("detect-blocking-io", "", "Detect blocking IO (on|on-at-startup|off)")
                .completion(comp::fixed(&["on", "on-at-startup", "off"])),
        )
        .flag(
            Flag::string("timezone", "", "Timezone")
                .short('t')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("logging", "", "Logging: debug|info|warning|error|critical")
                .short('l')
                .completion(comp::fixed(&["debug", "info", "warning", "error", "critical"])),
        )
        .flag(
            Flag::bool("debug", false, "Enable debug mode")
                .no_opt_default("false")
                .completion(comp::bools()),
        )
        .flag(
            Flag::bool("debug-block", false, "Enable debug mode with blocking startup")
                .no_opt_default("false")
                .completion(comp::bools()),
        )
        .flag(
            Flag::bool("diagnostics", false, "Enable diagnostics mode")
                .no_opt_default("false")
                .completion(comp::bools()),
        )
        // Go registers no completion function for --auto-update.
        .flag(Flag::bool("auto-update", true, "Enable/disable supervisor auto update").no_opt_default("true"))
        .flag(
            Flag::string_array(
                "feature-flag",
                "Set a development feature flag (name=true|false). Use multiple times for multiple flags.",
            )
            .completion(feature_flag_completions()),
        )
        .run(run)
}

fn run(ctx: &Ctx) -> bool {
    debug_args(ctx);

    let mut options = Options::new();

    for name in ["hostname", "channel", "detect-blocking-io", "timezone", "logging"] {
        let val = ctx.string(name);
        if !val.is_empty() && ctx.changed(name) {
            options.insert(name.replace('-', "_"), Value::String(val));
        }
    }

    for name in ["debug", "debug-block", "diagnostics", "auto-update"] {
        bool_if_changed(ctx, &mut options, name, &name.replace('-', "_"));
    }

    let mut feature_flags = ctx.string_array("feature-flag");
    // pflag quirk: `GetStringArray` reads a single empty value (`--feature-flag ""`)
    // back as an empty list, so Go sends nothing instead of failing on the name.
    if feature_flags.len() == 1 && feature_flags[0].is_empty() {
        feature_flags.clear();
    }
    if !feature_flags.is_empty() {
        let mut flag_map = Options::new();
        for entry in &feature_flags {
            let (name, value) = match entry.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (entry.as_str(), None),
            };
            let name = name.trim();
            if name.is_empty() {
                output::print_error_string(&format!(
                    "invalid feature flag {}: name must not be empty",
                    gofmt::quote(entry)
                ));
                return false;
            }
            let val = match value {
                None => true,
                Some(value) => match parse_bool(value) {
                    Ok(val) => val,
                    Err(_) => {
                        output::print_error_string(&format!(
                            "invalid value for feature flag {}: {}, expected true or false",
                            gofmt::quote(name),
                            gofmt::quote(value)
                        ));
                        return false;
                    }
                },
            };
            flag_map.insert(name.to_string(), Value::Bool(val));
        }
        options.insert("feature_flags".to_string(), Value::Object(flag_map));
    }

    Call::post("supervisor", "options").execute(ctx, options)
}

/// Go: `featureFlagCompletions` — the known feature flags with their value
/// toggled. (Go iterates a map in random order; here the order is sorted.)
fn feature_flag_completions() -> Completer {
    comp::from_fn(|_, _| {
        let mut ret = Vec::new();
        let Some(data) = completions::api_data("supervisor", "info") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        let Some(flags) = data.get("feature_flags").and_then(Value::as_object) else {
            return (ret, Directive::NO_FILE_COMP);
        };
        for (name, val) in flags {
            let Some(current) = val.as_bool() else { continue };
            ret.push(format!("{name}={}\tcurrently {current}", !current));
        }
        (ret, Directive::NO_FILE_COMP)
    })
}
