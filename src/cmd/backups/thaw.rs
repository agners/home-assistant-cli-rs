use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("thaw")
        .aliases(&["th"])
        .short("Thaw supervisor after an external backup")
        .long(
            "
End a freeze initiated by the freeze command after an external backup or snapshot
has completed.",
        )
        .example(
            "
  ha backups thaw",
        )
        .run(Call::post("backups", "thaw").run())
}
