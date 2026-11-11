use serde_json::Value;

use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command, comp};

pub fn command() -> Command {
    Command::new("add [repository]")
        .aliases(&["set", "new"])
        .short("Add new repository to Home Assistant store")
        .long(
            "
Add new repository of apps to the Home Assistant store.
",
        )
        .example(
            "
ha store add https://github.com/home-assistant/addons-example
",
        )
        .valid_args(comp::no_file())
        .args(Args::Exact(1))
        .run(Call::post("store", "repositories").run_with(|ctx, options| {
            options.insert("repository".into(), Value::from(ctx.arg(0)));
        }))
}
