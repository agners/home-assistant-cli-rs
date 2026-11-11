use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("repair [slug]")
        .aliases(&["reset"])
        .short("Repair/reset repository from Home Assistant store")
        .long(
            "
Repair/reset a repository of apps that is missing from the store,
showing incorrect information, or otherwise working incorrectly.
",
        )
        .example(
            "
ha store repair 94cfad5a
",
        )
        .valid_args(super::repositories_completions())
        .args(Args::Exact(1))
        .run(
            Call::post("store", "repositories/{slug}/repair")
                .arg_param("slug", 0)
                .run(),
        )
}
