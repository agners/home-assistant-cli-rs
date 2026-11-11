//! The command tree: the root command, its global flags and one module per
//! top-level command group.
//!
//! Porting agents: only edit files inside your group's directory
//! (`src/cmd/<group>/`). The registration below is complete.

pub mod completions;
pub mod helpers;
pub mod logs;

pub mod apps;
pub mod audio;
pub mod authentication;
pub mod available_updates;
pub mod backups;
pub mod banner;
pub mod cli;
pub mod core;
pub mod dns;
pub mod docker;
pub mod factory_reset;
pub mod hardware;
pub mod host;
pub mod info;
pub mod jobs;
pub mod mounts;
pub mod multicast;
pub mod network;
pub mod observer;
pub mod os;
pub mod refresh_updates;
pub mod resolution;
pub mod security;
pub mod store;
pub mod supervisor;
pub mod time;

use std::io::IsTerminal;

use crate::cobra::{self, Command, Ctx, Flag, comp};
use crate::{client, config, debug, log, spinner};

/// The root command's `PersistentPreRun`. Groups that override
/// `persistent_pre_run` (e.g. to warn about deprecated aliases) must call
/// this at the end, like the Go code calls `rootCmd.PersistentPreRun`.
pub fn root_pre_run(ctx: &Ctx) {
    let cfg = config::get();

    // set loglevel if possible
    log::set_level_text(&cfg.log_level);

    client::set_raw_json(cfg.raw_json && ctx.name() != "__complete");

    // Only shows spinner output when we have a TTY
    if !ctx.bool("no-progress") && std::io::stdout().is_terminal() {
        // Write to Stderr, helps when redirecting output, e.g., to a file
        spinner::enable_output();
    }

    debug!("Debug flags",
        "apiToken" => cfg.api_token,
        "cfgFile" => cfg.config,
        "endpoint" => cfg.endpoint,
        "logLevel" => cfg.log_level,
        "noProgress" => cfg.no_progress,
        "rawJSON" => cfg.raw_json,
    );
}

/// The program name as invoked (Go: `path.Base(os.Args[0])`).
fn program_name() -> String {
    let arg0 = std::env::args_os()
        .next()
        .map(|a| a.to_string_lossy().into_owned())
        .unwrap_or_default();
    match arg0.trim_end_matches('/').rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => "ha".to_string(),
    }
}

/// Build the complete command tree.
pub fn root() -> Command {
    let root = Command::new(&program_name())
        .short("A small CLI program to control Home Assistant")
        .long(
            "
The Home Assistant CLI is a small and simple command line utility that allows
you to control and configure different aspects of Home Assistant",
        )
        .persistent_flag(
            Flag::string(
                "config",
                "",
                "Optional config file (default is $HOME/.homeassistant.yaml)",
            )
            .filename(&["yaml", "yml"]),
        )
        .persistent_flag(
            Flag::string(
                "endpoint",
                "",
                "Endpoint for Home Assistant Supervisor (default is 'supervisor')",
            )
            .completion(comp::no_file()),
        )
        .persistent_flag(
            Flag::string("log-level", "", "Log level (defaults to WARN)")
                .completion(comp::fixed(&["DEBUG", "INFO", "WARN", "ERROR"])),
        )
        .persistent_flag(
            Flag::string("api-token", "", "Home Assistant Supervisor API token").completion(comp::no_file()),
        )
        .persistent_flag(Flag::bool("raw-json", false, "Output raw JSON from the API").completion(comp::bools()))
        .persistent_flag(Flag::bool("no-progress", false, "Disable the progress spinner").completion(comp::bools()))
        .persistent_pre_run(root_pre_run)
        .persistent_post_run(|_| {
            if spinner::active() {
                spinner::stop();
            }
        })
        .subcommand(apps::command())
        .subcommand(audio::command())
        .subcommand(authentication::command())
        .subcommand(available_updates::command())
        .subcommand(backups::command())
        .subcommand(banner::command())
        .subcommand(cli::command())
        .subcommand(core::command())
        .subcommand(dns::command())
        .subcommand(docker::command())
        .subcommand(factory_reset::command())
        .subcommand(hardware::command())
        .subcommand(host::command())
        .subcommand(info::command())
        .subcommand(jobs::command())
        .subcommand(mounts::command())
        .subcommand(multicast::command())
        .subcommand(network::command())
        .subcommand(observer::command())
        .subcommand(os::command())
        .subcommand(refresh_updates::command())
        .subcommand(resolution::command())
        .subcommand(security::command())
        .subcommand(store::command())
        .subcommand(supervisor::command())
        .subcommand(time::command());
    cobra::with_builtins(root, config::refresh)
}
