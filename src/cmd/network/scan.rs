use super::interface_completions;
use crate::cmd::helpers::Call;
use crate::cobra::{Args, Command};

pub fn command() -> Command {
    Command::new("scan [interface]")
        .aliases(&["accesspoints", "wifi"])
        .short("Scan for Access Points on a wireless interface.")
        .long(
            "
Scan for Access Points on a specific wireless interface.
This function works only on a wireless interface!
",
        )
        .example(
            "
  ha network scan wlan0
",
        )
        .valid_args(interface_completions())
        .args(Args::Exact(1))
        .run(
            Call::get("network", "interface/{interface}/accesspoints")
                .arg_param("interface", 0)
                .spinner()
                .run(),
        )
}
