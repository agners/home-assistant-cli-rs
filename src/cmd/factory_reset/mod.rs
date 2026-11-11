//! `ha factory-reset` (Go: `cmd/factory_reset.go`): `ha os datadisk wipe`
//! under another name.

use crate::cmd::helpers::leaf;
use crate::cmd::os::datadisk::wipe;
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("factory-reset")
        .short("Wipe all Home Assistant user data (alias for 'ha os datadisk wipe')")
        .long(wipe::LONG)
        .example(
            "
  ha factory-reset
",
        )
        .run(wipe::run)
}
