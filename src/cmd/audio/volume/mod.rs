//! `ha audio volume` (Go: `cmd/audio_volume*.go`).

mod input;
mod output;

use serde_json::Value;

use crate::client::Options;
use crate::cmd::helpers::{Call, debug_args};
use crate::cobra::{Command, Completer, Ctx, Directive, Flag, comp};

pub fn command() -> Command {
    Command::new("volume")
        .aliases(&["vol", "sound", "snd"])
        .short("Audio device volume control.")
        .long(
            "
Control the volume of your audio devices.
",
        )
        .example(
            "
\tha audio volume input --index 1 --mute
\tha audio volume input --index 2 --volume 75
\tha audio volume output --index 3 --unmute
\tha audio volume output --index 4 --volume 75 --application
",
        )
        .subcommand(input::command())
        .subcommand(output::command())
}

/// Go: `volumePercentCompletions` — `0` .. `100`.
fn volume_percent_completions() -> Completer {
    comp::from_fn(|_, _| ((0..=100).map(|i| i.to_string()).collect(), Directive::NO_FILE_COMP))
}

/// The flags shared by `audio volume input` and `audio volume output`.
fn volume_flags(cmd: Command) -> Command {
    cmd.flag(Flag::bool("application", false, "The index provided is an application").completion(comp::bools()))
        .flag(
            Flag::int("index", 0, "Channel index number")
                .required()
                .completion(comp::no_file()),
        )
        .flag(Flag::int("volume", 0, "Volume level in a percentage").completion(volume_percent_completions()))
        .flag(Flag::bool("mute", false, "Mute the channel").completion(comp::bools()))
        .flag(Flag::bool("unmute", false, "Unmute the channel").completion(comp::bools()))
}

/// `Run` of `audio volume input` / `output` (`direction`): sets the volume,
/// or (un)mutes when `--mute` / `--unmute` is true, of a channel or an
/// application.
fn run_volume(direction: &'static str) -> impl Fn(&Ctx) -> bool {
    move |ctx| {
        debug_args(ctx);

        let mut options = Options::new();

        let volume = ctx.int("volume");
        if volume != 0 && ctx.changed("volume") {
            options.insert("volume".to_string(), Value::from(volume as f64 / 100.0));
        }

        if ctx.changed("index") {
            options.insert("index".to_string(), Value::from(ctx.int("index")));
        }

        let mute = ctx.bool("mute");
        if ctx.changed("mute") {
            options.insert("active".to_string(), Value::Bool(mute));
        }

        let unmute = ctx.bool("unmute");
        if ctx.changed("unmute") {
            options.insert("active".to_string(), Value::Bool(!unmute));
        }

        let application = ctx.bool("application");
        let command = if (mute || unmute) && application {
            format!("mute/{direction}/application")
        } else if mute || unmute {
            format!("mute/{direction}")
        } else if application {
            format!("volume/{direction}/application")
        } else {
            format!("volume/{direction}")
        };

        Call::post("audio", &command).execute(ctx, options)
    }
}
