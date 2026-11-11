//! Configuration resolution, equivalent to how the Go CLI uses viper:
//!
//! 1. command line flag (only if explicitly set)
//! 2. environment: `SUPERVISOR_<KEY>` (`-` -> `_`); for `api-token` first
//!    `SUPERVISOR_API_TOKEN`, then `SUPERVISOR_TOKEN`. Empty variables count
//!    as unset.
//! 3. config file: `--config <file>` or `$HOME/.homeassistant.{json,yaml,yml}`
//! 4. defaults: endpoint `supervisor`, log-level `warn`
//!
//! Like in the Go CLI, the config *file* location only comes from the
//! `--config` flag (not from `SUPERVISOR_CONFIG`), and the spinner is only
//! disabled by the `--no-progress` flag itself (not by env or config file).

use std::collections::HashMap;
use std::sync::RwLock;

use crate::cobra::FlagSet;
use crate::cobra::flags::parse_bool;
use crate::{debug, info, log, output};

/// Resolved configuration values.
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub config: String,
    pub endpoint: String,
    pub log_level: String,
    pub api_token: String,
    pub raw_json: bool,
    pub no_progress: bool,
    /// `--no-progress` given on the command line (what the spinner honours).
    pub no_progress_flag: bool,
    /// Values read from the config file (lower-cased keys).
    file: HashMap<String, String>,
}

static CONFIG: RwLock<Option<Config>> = RwLock::new(None);

/// The current configuration (defaults if [`load`] has not run yet).
pub fn get() -> Config {
    if let Ok(guard) = CONFIG.read()
        && let Some(config) = guard.as_ref()
    {
        return config.clone();
    }
    resolve(&FlagSet::default(), HashMap::new())
}

/// `viper.GetString("endpoint")`
pub fn endpoint() -> String {
    get().endpoint
}

/// `viper.GetString("api-token")`
pub fn api_token() -> String {
    get().api_token
}

#[cfg(test)]
pub fn set_endpoint_for_test(endpoint: &str) {
    let mut config = resolve(&FlagSet::default(), HashMap::new());
    config.endpoint = endpoint.to_string();
    *CONFIG.write().unwrap() = Some(config);
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

/// One key: flag (if changed) > env > config file > default.
fn lookup(flags: &FlagSet, file: &HashMap<String, String>, key: &str, default: &str) -> String {
    if let Some(flag) = flags.lookup(key)
        && flag.changed
    {
        return flag.value.clone();
    }
    let env_key = format!("SUPERVISOR_{}", key.to_uppercase().replace('-', "_"));
    if let Some(v) = env(&env_key) {
        return v;
    }
    if key == "api-token"
        && let Some(v) = env("SUPERVISOR_TOKEN")
    {
        return v;
    }
    if let Some(v) = file.get(key) {
        return v.clone();
    }
    default.to_string()
}

fn to_bool(text: &str) -> bool {
    parse_bool(text).unwrap_or(false)
}

fn resolve(flags: &FlagSet, file: HashMap<String, String>) -> Config {
    Config {
        config: lookup(flags, &file, "config", ""),
        endpoint: lookup(flags, &file, "endpoint", "supervisor"),
        log_level: lookup(flags, &file, "log-level", "warn"),
        api_token: lookup(flags, &file, "api-token", ""),
        raw_json: to_bool(&lookup(flags, &file, "raw-json", "false")),
        no_progress: to_bool(&lookup(flags, &file, "no-progress", "false")),
        no_progress_flag: flags.get_bool("no-progress"),
        file,
    }
}

fn unquote(value: &str) -> String {
    let v = value.trim();
    if let Some(rest) = v.strip_prefix('"') {
        // Double-quoted scalar: up to the closing quote, with escapes.
        let mut out = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other),
                    None => {}
                },
                c => out.push(c),
            }
        }
        return out;
    }
    if let Some(rest) = v.strip_prefix('\'') {
        // Single-quoted scalar: '' is an escaped quote.
        let mut out = String::new();
        let mut chars = rest.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\'' {
                if chars.peek() == Some(&'\'') {
                    chars.next();
                    out.push('\'');
                } else {
                    break;
                }
            } else {
                out.push(c);
            }
        }
        return out;
    }
    // Plain scalar: strip a trailing comment.
    if v.starts_with('#') {
        return String::new();
    }
    match v.find(" #") {
        Some(i) => v[..i].trim_end().to_string(),
        None => v.to_string(),
    }
}

/// Parse the flat `key: value` YAML the CLI config file consists of.
fn parse_yaml(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        if line.starts_with([' ', '\t', '#', '-']) || line.trim().is_empty() {
            continue; // nested content, comments, document markers
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !value.is_empty() && !value.starts_with([' ', '\t']) {
            continue;
        }
        map.insert(unquote(key).to_lowercase(), unquote(value));
    }
    map
}

fn parse_json(text: &str) -> Option<HashMap<String, String>> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let mut map = HashMap::new();
    for (key, value) in value.as_object()? {
        let text = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => continue,
            other => other.to_string(),
        };
        map.insert(key.to_lowercase(), text);
    }
    Some(map)
}

fn read_file(path: &str) -> Option<HashMap<String, String>> {
    let text = std::fs::read_to_string(path).ok()?;
    match path.rsplit('.').next() {
        Some("yaml" | "yml") => Some(parse_yaml(&text)),
        Some("json") => parse_json(&text),
        _ => None,
    }
}

/// The Go CLI's `initConfig`: read env and config file, publish the result.
pub fn load(flags: &FlagSet) {
    // set loglevel if possible (before the config file is known)
    log::set_level_text(&lookup(flags, &HashMap::new(), "log-level", "warn"));

    let cfg_file = flags.get_string("config");
    let mut found: Option<(String, HashMap<String, String>)> = None;
    if !cfg_file.is_empty() {
        // Use config file from the flag.
        found = read_file(&cfg_file).map(|map| (cfg_file.clone(), map));
    } else {
        let Some(home) = env("HOME") else {
            output::print_error_string("Error while finding home directory: $HOME is not defined");
            std::process::exit(1);
        };
        debug!("Adding homedir to searchpath", "homedir" => home);
        for ext in ["json", "yaml", "yml"] {
            let path = format!("{home}/.homeassistant.{ext}");
            if std::path::Path::new(&path).is_file() {
                found = read_file(&path).map(|map| (path, map));
                break;
            }
        }
    }
    let file = match found {
        Some((path, map)) => {
            info!("Using configfile", "configfile" => path);
            map
        }
        None => {
            info!("No configfile found");
            HashMap::new()
        }
    };
    *CONFIG.write().unwrap() = Some(resolve(flags, file));
}

/// Re-resolve with another set of parsed flags, keeping the config file
/// values (used by shell completion, where flags are parsed late).
pub fn refresh(flags: &FlagSet) {
    let file = get().file;
    *CONFIG.write().unwrap() = Some(resolve(flags, file));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_yaml() {
        let map = parse_yaml(
            "# comment\n---\nendpoint: 192.168.1.2:8123  # dev box\nAPI-Token: \"abc def\"\nraw-json: true\nlog-level: 'debug'\nnested:\n  key: value\nempty:\n",
        );
        assert_eq!(map["endpoint"], "192.168.1.2:8123");
        assert_eq!(map["api-token"], "abc def");
        assert_eq!(map["raw-json"], "true");
        assert_eq!(map["log-level"], "debug");
        assert_eq!(map["empty"], "");
        let map = parse_yaml("a: \"x # y\"  # c\nb: 'it''s' # c\nc: # only comment\n");
        assert_eq!(
            (map["a"].as_str(), map["b"].as_str(), map["c"].as_str()),
            ("x # y", "it's", "")
        );
        assert!(!map.contains_key("key"));
    }

    #[test]
    fn json_file() {
        let map = parse_json(r#"{"endpoint": "host", "raw-json": true, "x": null}"#).unwrap();
        assert_eq!(map["endpoint"], "host");
        assert_eq!(map["raw-json"], "true");
        assert!(!map.contains_key("x"));
    }
}
