//! `ha os datadisk` (Go: `cmd/os_datadisk*.go`).

mod list;
mod move_disk;
pub mod wipe;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("datadisk")
        .aliases(&["data"])
        .short("Operating System DataDisk feature for managing data partition")
        .long(
            "
This command set is specifically designed for the Home Assistant Operating System
and only works on those systems. It provides an interface to get information
or migrating the data partition of the system to a different harddrive.",
        )
        .example(
            "
  ha os datadisk list
  ha os datadisk move /dev/sda1",
        )
        .subcommand(list::command())
        .subcommand(move_disk::command())
        .subcommand(wipe::command())
}
