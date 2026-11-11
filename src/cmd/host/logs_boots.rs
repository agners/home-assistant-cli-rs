//! Go: `cmd/host_logs_boots.go`. `hostBootCompletions` is
//! `crate::cmd::completions::host_boot()`.

use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("boots")
        .aliases(&["list-boots", "lb"])
        .short("Show all boot IDs by offset")
        .long(
            "
Show all values that can be used with the boot arg to find logs.
",
        )
        .example(
            "
  ha host logs boots
",
        )
        .run(Call::get("host", "logs/boots").run())
}
