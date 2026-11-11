use serde_json::Value;

use crate::client::Options;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, Flag, comp};
use crate::{debug, gofmt};

pub fn command() -> Command {
    Command::new("add [host]")
        .aliases(&["set", "new"])
        .short("Add new docker registry login for specific host")
        .long(
            "
Add new login for the Docker OCI registry server.
",
        )
        .example(
            "
  ha docker registries add my-docker.example.com --username \"test\" --password \"example\"
",
        )
        .valid_args(comp::no_file())
        .args(Args::Exact(1))
        .flag(
            Flag::string("username", "", "Username for OCI auth")
                .short('u')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("password", "", "Password for OCI auth")
                .short('p')
                .completion(comp::no_file()),
        )
        .run(|ctx| {
            debug!("registries add", "args" => gofmt::list(&ctx.args));

            // The body is `{"<host>": {...}}`, sent even when no credentials
            // are given (the outer map is never empty).
            let mut credentials = Options::new();
            for value in ["username", "password"] {
                let val = ctx.string(value);
                if !val.is_empty() && ctx.changed(value) {
                    credentials.insert(value.into(), Value::String(val));
                }
            }
            let mut options = Options::new();
            options.insert(ctx.arg(0).to_string(), Value::Object(credentials));

            Call::post("docker", "registries").execute(ctx, options)
        })
}
