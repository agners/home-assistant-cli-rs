//! `ha os datadisk move` (Go: `cmd/os_datadisk_move.go`; `move` is a Rust keyword).

use serde_json::Value;

use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, Directive, comp};

pub fn command() -> Command {
    Command::new("move [disk]")
        .aliases(&["migrate", "mov"])
        .short("Migrate Home Assistant Operating-System data partition")
        .long(
            "
This commands triggers an migration of the Home Assistant Operating-System
data partition to a new harddisk. The system reboots afterwards!
",
        )
        .example(
            "
  ha os datadisk move /dev/sda
",
        )
        .valid_args(comp::from_fn(|ctx, to_complete| {
            if !ctx.args.is_empty() {
                return (Vec::new(), Directive::NO_FILE_COMP);
            }
            if to_complete.is_empty() {
                return (vec!["/dev/".to_string()], Directive::NO_SPACE);
            }
            (Vec::new(), Directive::DEFAULT)
        }))
        .args(Args::Exact(1))
        .run(Call::post("os", "datadisk/move").run_with(|ctx, options| {
            options.insert("device".to_string(), Value::from(ctx.arg(0)));
        }))
}
