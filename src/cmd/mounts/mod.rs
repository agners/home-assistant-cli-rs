//! `ha mounts` — mounts in Supervisor (Go: `cmd/mounts*.go`).

mod add;
mod delete;
mod info;
mod options;
mod reload;
mod update;

use serde_json::Value;

use crate::client::{self, Options};
use crate::cmd::completions::describe;
use crate::cobra::{Command, Completer, Ctx, Directive, Flag, comp};
use crate::{debug, gofmt, output};

pub fn command() -> Command {
    Command::new("mounts")
        .aliases(&["mount", "mnts", "mnt"])
        .short("Get information, update or configure mounts in Supervisor")
        .long(
            "
The mounts command allows you to manage mounts in Supervisor by exposing
commands to view, mount, update or remove mounts such as network shares.",
        )
        .example(
            "
  ha mounts info
  ha mounts add my_share --usage media --type cifs --server server.local --share media",
        )
        .subcommand(add::command())
        .subcommand(delete::command())
        .subcommand(info::command())
        .subcommand(options::command())
        .subcommand(reload::command())
        .subcommand(update::command())
}

/// Go: `addMountFlags`. `type_default` / `usage_default` are the defaults of
/// `--type` / `--usage` (`mounts update` clears them and marks both flags
/// required).
fn add_mount_flags(cmd: Command, type_default: &str, usage_default: &str, required: bool) -> Command {
    let mut type_flag = Flag::string("type", type_default, "Type of mount")
        .short('t')
        .completion(comp::fixed(&["cifs", "nfs", "disk"]));
    let mut usage_flag = Flag::string("usage", usage_default, "Usage of mount within Home Assistant")
        .short('u')
        .completion(comp::fixed(&["backup", "media", "share"]));
    if required {
        type_flag = type_flag.required();
        usage_flag = usage_flag.required();
    }
    cmd.flag(type_flag)
        .flag(usage_flag)
        .flag(
            Flag::string("server", "", "IP address or hostname of network share server")
                .short('s')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::int(
                "port",
                0,
                "Port to use if network share is exposed on non-standard port for the type",
            )
            .short('o')
            .completion(comp::no_file()),
        )
        .flag(
            Flag::string("share", "", "Share to mount (cifs mounts only)")
                .short('r')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("username", "", "Username to use for authentication (cifs mounts only)")
                .short('n')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("password", "", "Password to use for authentication (cifs mounts only)")
                .short('p')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("version", "", "Version to use for the mount (cifs mounts only)")
                .short('v')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("path", "", "Path to mount (nfs mounts only)")
                .short('a')
                .completion(comp::no_file()),
        )
        .flag(
            Flag::string("device", "", "Device to mount, e.g. /dev/sda1 (disk mounts only)")
                .completion(mounts_device_completions()),
        )
        .flag(
            Flag::string("uuid", "", "Filesystem UUID of the device to mount (disk mounts only)")
                .completion(comp::no_file()),
        )
        .flag(
            Flag::bool(
                "read-only",
                false,
                "Is mount read-only (not available for backup mounts)",
            )
            .no_opt_default("true")
            .completion(comp::bools()),
        )
}

/// Go: `mountFlagsToOptions`. Copy only flags the user set. Sending an
/// untouched default would rewrite type or usage on update.
fn mount_flags_to_options(ctx: &Ctx, options: &mut Options) {
    for value in [
        "type", "usage", "server", "share", "path", "username", "password", "version", "device", "uuid",
    ] {
        let val = ctx.string(value);
        if !val.is_empty() && ctx.changed(value) {
            options.insert(value.to_string(), Value::String(val));
        }
    }

    let val = ctx.int("port");
    if val > 0 && ctx.changed("port") {
        options.insert("port".to_string(), Value::from(val));
    }

    if ctx.changed("read-only") {
        options.insert("read_only".to_string(), Value::Bool(ctx.bool("read-only")));
    }
}

/// Go: `mountFlagsToNewOptions`. Like `mount_flags_to_options`, but include
/// type/usage defaults for a new mount.
fn mount_flags_to_new_options(ctx: &Ctx, options: &mut Options) {
    mount_flags_to_options(ctx, options);

    for value in ["type", "usage"] {
        if options.contains_key(value) {
            continue;
        }
        let val = ctx.string(value);
        if !val.is_empty() {
            options.insert(value.to_string(), Value::String(val));
        }
    }
}

/// The shared tail of `add`, `delete`, `reload` and `update`: send `request`
/// (with the body if there are options) and show the response.
fn send(ctx: &Ctx, method: &str, command: &str, mut request: client::Request, options: Options) -> bool {
    let url = match client::url_helper("mounts", command) {
        Ok(url) => url,
        Err(err) => {
            output::print_error(&err);
            return false;
        }
    };
    if command.contains("{name}") {
        request = request.path_param("name", ctx.arg(0));
    }
    if !options.is_empty() {
        debug!("Request body", "options" => gofmt::json_marshal(&Value::Object(options.clone())));
        request = request.body(options);
    }
    let result = request.execute(method, &url);
    output::show(client::generic_json_error_handling(result))
}

/// Go: `mountsCompletions` — mount names with their details.
fn mounts_completions() -> Completer {
    comp::from_fn(|ctx, _| {
        let mut ret = Vec::new();
        if !ctx.args.is_empty() {
            return (ret, Directive::NO_FILE_COMP);
        }
        let Ok(resp) = client::generic_json_get("mounts", "") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        if !resp.is_success() || resp.data.result != "ok" {
            return (ret, Directive::NO_FILE_COMP);
        }
        for mount in resp
            .data
            .data
            .get("mounts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(m) = mount.as_object() else { continue };
            let Some(name) = m.get("name").and_then(Value::as_str) else {
                continue;
            };
            ret.push(describe(
                name,
                m,
                &["state", "usage", "server", "share", "path", "uuid", "filesystem"],
            ));
        }
        (ret, Directive::NO_FILE_COMP)
    })
}

/// Go: `mountsDeviceCompletions` — completions for `--device` from the host
/// disks. Empty or 404 yields none.
fn mounts_device_completions() -> Completer {
    comp::from_fn(|_, _| {
        let mut ret = Vec::new();
        let Ok(resp) = client::generic_json_get("host", "disks") else {
            return (ret, Directive::NO_FILE_COMP);
        };
        if !resp.is_success() || resp.data.result != "ok" {
            return (ret, Directive::NO_FILE_COMP);
        }
        for disk in resp
            .data
            .data
            .get("disks")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(d) = disk.as_object() else { continue };
            for partition in d.get("partitions").and_then(Value::as_array).into_iter().flatten() {
                let Some(p) = partition.as_object() else { continue };
                if p.get("mountable").and_then(Value::as_bool) != Some(true) {
                    continue;
                }
                let Some(device) = p.get("device").and_then(Value::as_str).filter(|s| !s.is_empty()) else {
                    continue;
                };
                let ds = mount_device_description(d, p);
                if ds.is_empty() {
                    ret.push(device.to_string());
                } else {
                    ret.push(format!("{device}\t{ds}"));
                }
            }
        }
        (ret, Directive::NO_FILE_COMP)
    })
}

fn non_empty_str<'a>(m: &'a Options, key: &str) -> Option<&'a str> {
    m.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

/// Go: `mountDeviceDescription`
fn mount_device_description(disk: &Options, partition: &Options) -> String {
    let mut ds: Vec<String> = Vec::new();
    if let Some(s) = non_empty_str(disk, "name") {
        ds.push(s.to_string());
    } else {
        let name: Vec<&str> = ["vendor", "model"]
            .iter()
            .filter_map(|key| non_empty_str(disk, key))
            .collect();
        if !name.is_empty() {
            ds.push(name.join(" "));
        }
    }
    if let Some(s) = non_empty_str(partition, "label") {
        ds.push(s.to_string());
    }
    if let Some(size) = partition.get("size").and_then(Value::as_f64).filter(|s| *s > 0.0) {
        ds.push(humanize_mount_size(size));
    }
    ds.join(", ")
}

/// Go: `humanizeMountSize`. Decimal units, matching how drive capacity is
/// labelled.
fn humanize_mount_size(mut size: f64) -> String {
    let units = ["B", "kB", "MB", "GB", "TB"];
    let mut i = 0;
    while size >= 1000.0 && i < units.len() - 1 {
        size /= 1000.0;
        i += 1;
    }
    if i == 0 {
        format!("{size:.0} {}", units[i])
    } else {
        format!("{size:.1} {}", units[i])
    }
}

#[cfg(test)]
mod tests {
    use super::humanize_mount_size;

    #[test]
    fn humanize() {
        assert_eq!(humanize_mount_size(512.0), "512 B");
        assert_eq!(humanize_mount_size(1000.0), "1.0 kB");
        assert_eq!(humanize_mount_size(32_000_000_000.0), "32.0 GB");
        assert_eq!(humanize_mount_size(5e15), "5000.0 TB");
    }
}
