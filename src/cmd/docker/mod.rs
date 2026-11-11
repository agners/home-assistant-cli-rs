//! `ha docker` — Docker backend information and configuration
//! (Go: `cmd/docker*.go`).

mod info;
mod migrate_storage_driver;
mod options;
mod registries;
mod registries_add;
mod registries_delete;
mod reset_storage;

use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("docker")
        .aliases(&["do"])
        .short("Docker backend specific for info and OCI configuration")
        .long(
            "
The docker command provides command-line tools to control the host docker that
Home Assistant is running on. It allows you to do things like use private OCI registries.",
        )
        .example(
            "
  ha docker info
  ha docker options
  ha docker registries
  ha docker reset-storage",
        )
        .subcommand(info::command())
        .subcommand(migrate_storage_driver::command())
        .subcommand(options::command())
        .subcommand(registries::command())
        .subcommand(reset_storage::command())
}
