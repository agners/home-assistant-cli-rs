//! Minimal structured logging to stderr in the format of Go's `log/slog`
//! default handler: `2006/01/02 15:04:05 LEVEL message key=value ...`.

use std::sync::atomic::{AtomicI32, Ordering};

pub const DEBUG: i32 = -4;
pub const INFO: i32 = 0;
pub const WARN: i32 = 4;
pub const ERROR: i32 = 8;

static LEVEL: AtomicI32 = AtomicI32::new(WARN);

/// `slog.Level.UnmarshalText`: `DEBUG`, `info`, `Warn+2`, `ERROR-1`, ...
pub fn parse_level(text: &str) -> Option<i32> {
    let (name, offset) = match text.find(['+', '-']) {
        Some(i) => (&text[..i], text[i..].parse::<i32>().ok()?),
        None => (text, 0),
    };
    let base = match name.to_ascii_uppercase().as_str() {
        "DEBUG" => DEBUG,
        "INFO" => INFO,
        "WARN" => WARN,
        "ERROR" => ERROR,
        _ => return None,
    };
    Some(base + offset)
}

/// Set the log level from text; unknown text leaves the level unchanged.
pub fn set_level_text(text: &str) {
    if let Some(level) = parse_level(text) {
        LEVEL.store(level, Ordering::Relaxed);
    }
}

pub fn enabled(level: i32) -> bool {
    level >= LEVEL.load(Ordering::Relaxed)
}

fn level_name(level: i32) -> String {
    let (name, base) = match level {
        l if l < INFO => ("DEBUG", DEBUG),
        l if l < WARN => ("INFO", INFO),
        l if l < ERROR => ("WARN", WARN),
        _ => ("ERROR", ERROR),
    };
    match level - base {
        0 => name.to_string(),
        d => format!("{name}{d:+}"),
    }
}

fn timestamp() -> String {
    // SAFETY: `time` and `localtime_r` are called with valid pointers to
    // local storage.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut tm);
        format!(
            "{:04}/{:02}/{:02} {:02}:{:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    }
}

fn needs_quoting(s: &str) -> bool {
    s.is_empty() || s.chars().any(|c| c == ' ' || c == '"' || c == '=' || c.is_control())
}

/// Write a log record if `level` is enabled.
pub fn log(level: i32, message: &str, attrs: &[(&str, String)]) {
    if !enabled(level) {
        return;
    }
    let mut line = format!("{} {} {}", timestamp(), level_name(level), message);
    for (key, value) in attrs {
        line.push(' ');
        line.push_str(key);
        line.push('=');
        if needs_quoting(value) {
            line.push_str(&crate::gofmt::quote(value));
        } else {
            line.push_str(value);
        }
    }
    eprintln!("{line}");
}

/// `slog.Debug(msg, key, value, ...)`: `debug!("dns info", "args" => list)`.
#[macro_export]
macro_rules! debug {
    ($msg:expr $(, $key:expr => $value:expr)* $(,)?) => {
        if $crate::log::enabled($crate::log::DEBUG) {
            $crate::log::log($crate::log::DEBUG, $msg, &[$(($key, format!("{}", $value))),*]);
        }
    };
}

/// `slog.Info(msg, key, value, ...)`
#[macro_export]
macro_rules! info {
    ($msg:expr $(, $key:expr => $value:expr)* $(,)?) => {
        if $crate::log::enabled($crate::log::INFO) {
            $crate::log::log($crate::log::INFO, $msg, &[$(($key, format!("{}", $value))),*]);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels() {
        assert_eq!(parse_level("debug"), Some(DEBUG));
        assert_eq!(parse_level("WARN"), Some(WARN));
        assert_eq!(parse_level("Info+2"), Some(2));
        assert_eq!(parse_level("error-1"), Some(7));
        assert_eq!(parse_level("verbose"), None);
        assert_eq!(parse_level(""), None);
        assert_eq!(level_name(2), "INFO+2");
    }
}
