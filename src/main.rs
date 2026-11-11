//! `ha` — the Home Assistant CLI, a Rust port of github.com/home-assistant/cli.

// The cobra-style framework and client keep a few API pieces (PUT/query
// helpers, extra arg validators and completion directives) that no command
// uses today.
#![allow(dead_code)]

mod client;
mod cmd;
mod cobra;
mod config;
mod gofmt;
mod goyaml;
mod log;
mod output;
mod prompt;
mod spinner;

use std::io::Write;

fn main() {
    // Die silently on a closed pipe (`ha host logs | head`) like the Go
    // binary does, instead of panicking on a failed write.
    // SAFETY: resetting a signal disposition to the default is always sound.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let root = cmd::root();
    let success = match cobra::execute(&root, &args, &config::load) {
        Ok(success) => success,
        Err(message) => {
            // cobra already reported the error; Execute() prints it again.
            output::print_error_string(&message);
            false
        }
    };
    let _ = std::io::stdout().flush();
    std::process::exit(if success { 0 } else { 1 });
}
