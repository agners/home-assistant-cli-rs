use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("healthcheck")
        .aliases(&["evaluate", "run"])
        .short("Execute system healthcheck and fixups")
        .long(
            "
This command execute a full system check and auto fixups. It check all issues again to see if they
are still around and try to fix it again.",
        )
        .example(
            "
  ha resolution healthcheck",
        )
        .run(Call::post("resolution", "healthcheck").spinner().run())
}
