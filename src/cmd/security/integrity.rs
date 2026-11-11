use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("integrity")
        .aliases(&["int", "trust"])
        .short("Execute security integrity check")
        .long(
            "
This command execute a full system integrity check.
This need content trust to be enabled.",
        )
        .example(
            "
  ha security integrity",
        )
        .run(Call::post("security", "integrity").spinner().run())
}
