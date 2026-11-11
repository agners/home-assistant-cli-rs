use serde_json::Value;

use super::{boot_slot_completions, get_boot_slots};
use crate::client::{self, Options};
use crate::cmd::helpers::{Call, debug_args};
use crate::cobra::{Args, Command};
use crate::output;

pub fn command() -> Command {
    Command::new("boot-slot [slot]")
        .aliases(&["boot"])
        .short("Changes the active boot slot")
        .long(
            "
Using this command you can change the active boot slot to rollback
an OS update without making more changes to the system.
",
        )
        .example(
            "
  ha os boot-slot other
  ha os boot-slot A
",
        )
        .valid_args(boot_slot_completions())
        .args(Args::Exact(1))
        .run(|ctx| {
            debug_args(ctx);

            let mut boot_slot = ctx.arg(0).to_string();
            // The Go client keeps the timeout of the first request of the
            // process, so the reboot timeout only applies when the slot does
            // not have to be looked up first.
            let mut timeout = client::REBOOT_TIMEOUT;
            if boot_slot == "other" {
                timeout = client::DEFAULT_TIMEOUT;
                let boot_slots = match get_boot_slots() {
                    Ok(Some(boot_slots)) => boot_slots,
                    Ok(None) => return false,
                    Err(err) => {
                        output::print_error(&err);
                        return false;
                    }
                };

                // Go iterates the slots in random map order; here sorted.
                for (name, v) in &boot_slots {
                    let Some(info) = v.as_object() else { continue };
                    if info.get("state").and_then(Value::as_str) == Some("inactive") {
                        boot_slot = name.clone();
                        break;
                    }
                }
                if boot_slot == "other" {
                    return false;
                }
            }

            let mut options = Options::new();
            options.insert("boot_slot".to_string(), Value::String(boot_slot));
            Call::post("os", "boot-slot").timeout(timeout).execute(ctx, options)
        })
}
