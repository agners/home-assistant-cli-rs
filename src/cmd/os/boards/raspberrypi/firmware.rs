use serde_json::Value;

use super::firmware_update;
use crate::client::{self, Options, Response};
use crate::cmd::helpers::{debug_args, leaf};
use crate::cobra::Command;
use crate::output;

/// Go: `getRPiFirmwareInfo` fetches the Raspberry Pi firmware status. A 404
/// is turned into a friendly error in case the API doesn't return it already.
pub(super) fn get_firmware_info() -> client::Result<Response> {
    let url = client::url_helper("os", "boards/raspberrypi/firmware")?;

    let resp = client::json_request_timeout(client::DEFAULT_TIMEOUT).get(&url)?;

    if resp.status_code() == 404 {
        // Prefer the Supervisor's own message when it returns a JSON body,
        // otherwise (e.g. a bare 404 from an older Supervisor) explain why.
        if !resp.data.message.is_empty() {
            return Err(client::Error::new(resp.data.message));
        }
        return Err(client::Error::new(
            "firmware information is not available on this system \
             (requires a Raspberry Pi 4 / 5 or Home Assistant Yellow with Home Assistant OS 18.0 or newer)",
        ));
    }

    client::generic_json_error_handling(Ok(resp))
}

/// `time.Unix(secs, 0).UTC().Format("2006-01-02")`
fn format_date(secs: i64) -> String {
    // Civil date from days since the Unix epoch (proleptic Gregorian).
    let days = i128::from(secs.div_euclid(86400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i128::from(month <= 2);
    if year < 0 {
        format!("-{:04}-{month:02}-{day:02}", -year)
    } else {
        format!("{year:04}-{month:02}-{day:02}")
    }
}

/// Go: `humanizeRPiFirmwareVersion` turns a raw firmware version into a
/// human-readable string. The Supervisor reports the bootloader EEPROM build
/// as a Unix timestamp, optionally suffixed with the VL805 EEPROM revision
/// (timestamp-hexstring). It renders the timestamp as a UTC YYYY-MM-DD date
/// and appends (VL805 hexstring) when a VL805 revision is present.
pub(super) fn humanize_version(version: &str) -> String {
    if version.is_empty() {
        return version.to_string();
    }
    let (timestamp, vl805) = version.split_once('-').unwrap_or((version, ""));
    let Ok(secs) = timestamp.parse::<i64>() else {
        return version.to_string();
    };
    let date = format_date(secs);
    if !vl805.is_empty() {
        return format!("{date} (VL805 {vl805})");
    }
    date
}

/// Go: `humanizeRPiFirmwareData` humanizes the version fields of a firmware
/// info response in place, leaving non-string values (e.g. null) untouched.
fn humanize_data(data: &mut Options) {
    for key in ["current_version", "latest_version"] {
        if let Some(Value::String(v)) = data.get_mut(key) {
            *v = humanize_version(v);
        }
    }
}

pub fn command() -> Command {
    leaf("firmware")
        .aliases(&["fw"])
        .short("Show Raspberry Pi firmware information")
        .long(
            "
This command shows information about the Raspberry Pi bootloader (EEPROM) firmware,
including the currently installed and latest bundled versions and whether an update
is available. Available on Raspberry Pi 4 / 5 and Home Assistant Yellow.",
        )
        .example(
            "
  ha os boards raspberrypi firmware",
        )
        .run(|ctx| {
            debug_args(ctx);

            match get_firmware_info() {
                Ok(mut resp) => output::show_json_response_transform(&mut resp, Some(&humanize_data)),
                Err(err) => {
                    output::print_error(&err);
                    false
                }
            }
        })
        .subcommand(firmware_update::command())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize() {
        assert_eq!(humanize_version(""), "");
        assert_eq!(humanize_version("1736899200"), "2025-01-15");
        assert_eq!(humanize_version("1736899200-000138c0"), "2025-01-15 (VL805 000138c0)");
        assert_eq!(humanize_version("1736899200-"), "2025-01-15");
        assert_eq!(humanize_version("abc"), "abc");
        assert_eq!(humanize_version("0"), "1970-01-01");
        assert_eq!(humanize_version("951782400"), "2000-02-29");
        assert_eq!(humanize_version("-1"), "-1");
    }
}
