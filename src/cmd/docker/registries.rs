use super::{registries_add, registries_delete};
use crate::cmd::helpers::Call;
use crate::cobra::Command;

pub fn command() -> Command {
    Command::new("registries")
        .aliases(&["reg", "re"])
        .short("Manage private OCI docker registry")
        .long(
            "
Manage private OCI registry server on the local Docker host.
",
        )
        .example(
            "
\tha docker registries
\tha docker registries add my-docker.example.com
\tha docker registries delete my-docker.example.com
",
        )
        .run(Call::get("docker", "registries").run())
        .subcommand(registries_add::command())
        .subcommand(registries_delete::command())
}
