//! `ha banner` (Go: `cmd/banner.go`).

use std::io::Write;
use std::time::Duration;

use serde_json::Value;

use crate::client::{self, Options};
use crate::cmd::helpers::leaf;
use crate::cobra::{Command, Flag, comp};
use crate::{debug, gofmt, output};

#[rustfmt::skip]
const HA_BANNER: &str = concat!(
    "\n",
    "       ▄██▄           _   _                                    \n",
    "     ▄██████▄        | | | | ___  _ __ ___   ___               \n",
    "   ▄████▀▀████▄      | |_| |/ _ \\| '_ ` _ \\ / _ \\              \n",
    " ▄█████    █████▄    |  _  | (_) | | | | | |  __/              \n",
    "▄██████▄  ▄██████▄   |_| |_|\\___/|_| |_| |_|\\___|          _   \n",
    "████████  ██▀  ▀██      / \\   ___ ___(_)___| |_ __ _ _ __ | |_ \n",
    "███▀▀███  ██   ▄██     / _ \\ / __/ __| / __| __/ _` | '_ \\| __|\n",
    "██    ██  ▀ ▄█████    / ___ \\\\__ \\__ \\ \\__ \\ || (_| | | | | |_ \n",
    "███▄▄ ▀█  ▄███████   /_/   \\_\\___/___/_|___/\\__\\__,_|_| |_|\\__|\n",
    "▀█████▄   ███████▀\n",
    "\n",
    "Welcome to the Home Assistant command line interface.\n",
);

/// Go: `supervisorGet` — the `data` object of a `result: ok` answer (`None`
/// if it is empty), an error for transport errors and API errors.
fn supervisor_get(section: &str, command: &str) -> client::Result<Option<Options>> {
    let resp = client::generic_json_get(section, command)?;
    let data = resp.data;
    if data.result == "ok" {
        Ok(if data.data.is_empty() { None } else { Some(data.data) })
    } else {
        Err(client::Error::new(format!(
            "error returned from Supervisor: {}",
            data.message
        )))
    }
}

/// The Go name of the dynamic type of a decoded JSON value.
fn go_type(v: Option<&Value>) -> &'static str {
    match v {
        None | Some(Value::Null) => "nil",
        Some(Value::Bool(_)) => "bool",
        Some(Value::Number(_)) => "float64",
        Some(Value::String(_)) => "string",
        Some(Value::Array(_)) => "[]interface {}",
        Some(Value::Object(_)) => "map[string]interface {}",
    }
}

/// The Go CLI crashes here (failed type assertion / nil dereference in
/// `cmd/banner.go`). Mirror the first lines of the Go runtime's report and
/// its exit code 2; the goroutine trace cannot be reproduced.
fn go_panic(message: &str) -> ! {
    let _ = std::io::stdout().flush();
    eprintln!("panic: {message}\n\ngoroutine 1 [running]:");
    std::process::exit(2);
}

fn conversion_panic(v: Option<&Value>, want: &str) -> ! {
    go_panic(&format!(
        "interface conversion: interface {{}} is {}, not {want}",
        go_type(v)
    ))
}

/// Go: `v.([]any)`
fn as_list(v: Option<&Value>) -> &Vec<Value> {
    match v {
        Some(Value::Array(list)) => list,
        _ => conversion_panic(v, "[]interface {}"),
    }
}

/// Go: `v.(map[string]any)`
fn as_map(v: Option<&Value>) -> &Options {
    match v {
        Some(Value::Object(map)) => map,
        _ => conversion_panic(v, "map[string]interface {}"),
    }
}

/// Go's `fmt` rendering of a decoded JSON value: `%v` (`fmt.Sprint`) or,
/// with `verb_s`, `%s` (which flags non-strings as `%!s(type=value)`).
fn go_format(v: Option<&Value>, verb_s: bool) -> String {
    match v {
        None | Some(Value::Null) if verb_s => "%!s(<nil>)".to_string(),
        None | Some(Value::Null) => "<nil>".to_string(),
        Some(Value::Bool(b)) if verb_s => format!("%!s(bool={b})"),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => {
            let f = gofmt::format_float_g(n.as_f64().unwrap_or_default());
            if verb_s { format!("%!s(float64={f})") } else { f }
        }
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(list)) => {
            let items: Vec<String> = list.iter().map(|v| go_format(Some(v), verb_s)).collect();
            format!("[{}]", items.join(" "))
        }
        Some(Value::Object(map)) => {
            let items: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{k}:{}", go_format(Some(v), verb_s)))
                .collect();
            format!("map[{}]", items.join(" "))
        }
    }
}

/// Go: `getAddresses`
fn get_addresses(addresses: &[Value]) -> String {
    let addresses: Vec<String> = addresses.iter().map(|v| go_format(Some(v), false)).collect();
    addresses.join(", ")
}

/// Go: `%-25s` (pads to 25 characters).
fn pad(s: &str) -> String {
    format!("{s:<25}")
}

fn run_banner(nowait: bool) -> bool {
    let mut netinfo: Option<Options> = None;
    if !nowait {
        let mut supervisor_ready = false;

        // 3 minutes timeout
        for i in 0..180 {
            netinfo = supervisor_get("network", "info").ok().flatten();
            if let Some(info) = &netinfo
                && let Some(netifaces) = info.get("interfaces")
                && !as_list(Some(netifaces)).is_empty()
            {
                println!("Home Assistant Supervisor is running!");
                supervisor_ready = true;
                break;
            }
            if i == 0 {
                println!("Waiting for Supervisor to start...");
            }
            std::thread::sleep(Duration::from_secs(1));
        }

        if !supervisor_ready {
            println!("Supervisor is taking longer than expected to start. Use 'ha supervisor logs' to check logs.");
            return true;
        }
    }

    println!("System information:");
    let netinfo = match netinfo {
        Some(netinfo) => netinfo,
        None => match supervisor_get("network", "info") {
            Ok(Some(netinfo)) => netinfo,
            // Go dereferences the nil data pointer of an empty answer.
            Ok(None) => go_panic(
                "runtime error: invalid memory address or nil pointer dereference\n\
                 [signal SIGSEGV: segmentation violation]",
            ),
            Err(err) => {
                println!("  Network information unavailable: {err}");
                return false;
            }
        },
    };

    // Print network address information
    if let Some(netifaces) = netinfo.get("interfaces") {
        for netiface in as_list(Some(netifaces)) {
            let nf = as_map(Some(netiface));
            let interface = go_format(nf.get("interface"), true);
            let title_ipv4 = format!("IPv4 addresses for {interface}:");
            let title_ipv6 = format!("IPv6 addresses for {interface}:");

            match nf.get("ipv4") {
                None | Some(Value::Null) => println!("  {} (No address)", pad(&title_ipv4)),
                ipv4 => {
                    let ipv4_addresses = as_list(as_map(ipv4).get("address"));
                    if ipv4_addresses.is_empty() {
                        println!("  {} (No address)", pad(&title_ipv4));
                    } else {
                        println!("  {} {}", pad(&title_ipv4), get_addresses(ipv4_addresses));
                    }
                }
            }

            match nf.get("ipv6") {
                None | Some(Value::Null) => {}
                ipv6 => {
                    let ipv6_addresses = as_list(as_map(ipv6).get("address"));
                    if !ipv6_addresses.is_empty() {
                        println!("  {} {}", pad(&title_ipv6), get_addresses(ipv6_addresses));
                    }
                }
            }
        }
    } else {
        print!("  (No networking information)");
    }
    println!();

    // Print Host URL
    let hostinfo = match supervisor_get("host", "info") {
        Ok(Some(hostinfo)) => hostinfo,
        Ok(None) => return true,
        Err(err) => {
            output::print_error(&err);
            return false;
        }
    };
    let coreinfo = match supervisor_get("core", "info") {
        Ok(Some(coreinfo)) => coreinfo,
        Ok(None) => return true,
        Err(err) => {
            output::print_error(&err);
            return false;
        }
    };

    let (protocol, default_port) = if coreinfo.get("ssl").and_then(Value::as_bool) == Some(true) {
        ("https", 443)
    } else {
        ("http", 80)
    };

    // Go: `int(port)` of the float64 (0 if it is not a number).
    let port = coreinfo.get("port").and_then(Value::as_f64).unwrap_or_default() as i64;
    let hostname = go_format(hostinfo.get("hostname"), true);
    let mut core_url = format!("{protocol}://{hostname}.local");
    if port != default_port {
        core_url = format!("{core_url}:{port}");
    }

    println!(
        "  {} {}",
        pad("OS Version:"),
        go_format(hostinfo.get("operating_system"), true)
    );
    println!(
        "  {} {}",
        pad("Home Assistant Core:"),
        go_format(coreinfo.get("version"), true)
    );
    println!();
    println!("  {} {}", pad("Home Assistant URL:"), core_url);
    println!("  {} http://{}.local:{}", pad("Observer URL:"), hostname, 4357);
    println!();
    println!("System is ready! Use browser or app to configure.");
    true
}

pub fn command() -> Command {
    leaf("banner")
        .aliases(&["ba"])
        .short("Prints the CLI Home Assistant banner along with some useful information")
        .example(
            "
  ha banner
\t",
        )
        .flag(Flag::bool("no-wait", false, "Don't wait until Supervisor is started").completion(comp::bools()))
        .run(|ctx| {
            debug!("info", "args" => gofmt::list(&ctx.args));

            print!("{HA_BANNER}");
            println!();

            run_banner(ctx.bool("no-wait"))
        })
}
