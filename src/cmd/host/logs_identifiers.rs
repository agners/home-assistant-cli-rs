use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("identifiers")
        .aliases(&["ids", "list-identifiers", "li"])
        .short("Show all syslog identifiers")
        .long(
            "
Show all values that can be used with the identifier arg to find logs.
",
        )
        .example(
            "
  ha host logs identifiers
",
        )
        .run(Call::get("host", "logs/identifiers").run())
}
