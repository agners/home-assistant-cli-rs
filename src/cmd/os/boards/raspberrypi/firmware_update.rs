use serde_json::Value;

use super::firmware::{get_firmware_info, humanize_version};
use crate::client::Options;
use crate::cmd::helpers::{Call, debug_args, leaf};
use crate::cobra::Command;
use crate::{output, prompt};

pub fn command() -> Command {
    leaf("update")
        .aliases(&["up", "upgrade"])
        .short("Apply the bundled Raspberry Pi firmware update")
        .long(
            "
This command applies the bundled Raspberry Pi firmware update (bootloader EEPROM,
and VL805 where present). A reboot is required for the new firmware to take effect.",
        )
        .example(
            "
  ha os boards raspberrypi firmware update",
        )
        .run(|ctx| {
            debug_args(ctx);

            // Fetch the current firmware status first so we can fail gracefully.
            let mut info_resp = match get_firmware_info() {
                Ok(resp) => resp,
                Err(err) => {
                    output::print_error(&err);
                    return false;
                }
            };
            if !info_resp.is_success() {
                // Other API errors (e.g. a 400) come back with a message to show.
                output::show_json_response(&mut info_resp);
                return false;
            }

            let info = &info_resp.data.data;
            let flag = |key: &str| info.get(key).and_then(Value::as_bool);
            let text = |key: &str| info.get(key).and_then(Value::as_str).unwrap_or_default();

            if flag("update_blocked") == Some(true) {
                let mut msg = "firmware update is blocked".to_string();
                let reason = text("blocked_reason");
                if !reason.is_empty() {
                    msg = format!("{msg}: {reason}");
                }
                output::print_error_string(&msg);
                return false;
            }

            if flag("update_pending") == Some(true) {
                println!(
                    "A firmware update is already pending. Reboot the device using `ha host reboot` to finish the installation."
                );
                return true;
            }

            let current = text("current_version");
            let latest = text("latest_version");

            if flag("update_available") == Some(false) {
                if current.is_empty() {
                    println!("The Raspberry Pi firmware is already up to date.");
                } else {
                    println!(
                        "The Raspberry Pi firmware is already up to date ({}).",
                        humanize_version(current)
                    );
                }
                return true;
            }

            let mut text = "This applies the bundled Raspberry Pi firmware update and requires a reboot to take effect."
                .to_string();
            if !current.is_empty() && !latest.is_empty() {
                text = format!(
                    "This will update the Raspberry Pi firmware from {} to {} and requires a reboot to take effect.",
                    humanize_version(current),
                    humanize_version(latest)
                );
            }

            match prompt::ask_for_confirmation(&format!("{text} Continue?"), 2) {
                Err(err) => {
                    eprintln!("Aborted: {err}");
                    return false;
                }
                Ok(false) => {
                    eprintln!("Aborted.");
                    return false;
                }
                Ok(true) => {}
            }

            println!("\nApplying firmware update. Please be patient and do not unplug or power off the device.");

            let success = Call::post("os", "boards/raspberrypi/firmware/update")
                .spinner()
                .execute(ctx, Options::new());
            if success {
                println!(
                    "\nFirmware update applied. Reboot the device using `ha host reboot` to finish the installation."
                );
            }
            success
        })
}
