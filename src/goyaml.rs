//! YAML rendering identical to what the Go CLI prints.
//!
//! The Go CLI renders API data with `ghodss/yaml`: the data map is marshalled
//! to JSON, re-read with go-yaml v2 (which re-types the scalars) and emitted
//! with go-yaml v2's port of the libyaml emitter. This module reproduces that
//! pipeline for a `serde_json::Value`:
//!
//! * mapping keys are sorted with go-yaml's "natural" key order
//! * numbers go through Go's float64 -> JSON text -> YAML int/float typing
//! * strings are quoted exactly when go-yaml would (`resolve` + libyaml
//!   scalar analysis), long scalars are folded at 80 columns, multi-line
//!   strings use literal block style
//! * sequences inside mappings are not indented

use std::cmp::Ordering;

use serde_json::Value;

use crate::cobra::flags::{parse_int, parse_uint};
use crate::gofmt::{format_float_g, format_float_json};

const BEST_INDENT: i32 = 2;
const BEST_WIDTH: usize = 80;

/// Render a JSON value the way `ghodss/yaml.JSONToYAML` would. Errors are
/// the ones go-yaml reports when it re-reads the JSON text (only possible
/// with unusual control characters in strings).
pub fn to_yaml(value: &Value) -> Result<String, String> {
    let value = reread_as_yaml(value)?;
    let mut e = Emitter {
        out: String::new(),
        column: 0,
        indent: -1,
        whitespace: true,
        indention: true,
        mapping_context: false,
        simple_key_context: false,
    };
    e.emit_node(&value, false, false);
    // document end
    e.write_indent();
    Ok(e.out)
}

// ---------------------------------------------------------------------------
// Side effects of go-yaml parsing the JSON text produced by json.Marshal
// ---------------------------------------------------------------------------

const NEL: char = '\u{85}';

/// Characters the YAML reader rejects when they appear raw in the JSON text
/// (json.Marshal escapes everything below U+0020, U+2028 and U+2029).
fn is_forbidden(c: char) -> bool {
    matches!(c as u32, 0x7F..=0x84 | 0x86..=0x9F | 0xFEFF | 0xFFFE | 0xFFFF)
}

/// YAML line folding of raw NEL line breaks inside a double-quoted scalar:
/// blanks around the break are dropped, one break becomes a space, `n`
/// breaks become `n - 1` newlines.
fn fold_nel(s: &str) -> String {
    if !s.contains(NEL) {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        // A run of spaces, possibly followed by a break.
        let mut j = i;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        if j < chars.len() && chars[j] == NEL {
            let mut breaks = 0;
            while j < chars.len() && (chars[j] == ' ' || chars[j] == NEL) {
                breaks += (chars[j] == NEL) as usize;
                j += 1;
            }
            if breaks == 1 {
                out.push(' ');
            } else {
                out.push_str(&"\n".repeat(breaks - 1));
            }
            i = j;
        } else if j > i {
            out.extend(&chars[i..j]);
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn reread_node(value: &Value, line: &mut usize) -> Result<Value, String> {
    Ok(match value {
        Value::String(s) => {
            *line += s.matches(NEL).count();
            Value::String(fold_nel(s))
        }
        Value::Array(items) => Value::Array(items.iter().map(|v| reread_node(v, line)).collect::<Result<_, _>>()?),
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, v) in map {
                if key.contains(NEL) {
                    // A multi-line key is not a valid simple key.
                    *line += key.matches(NEL).count();
                    return Err(format!("yaml: line {line}: did not find expected ',' or '}}'"));
                }
                out.insert(key.clone(), reread_node(v, line)?);
            }
            Value::Object(out)
        }
        other => other.clone(),
    })
}

fn has_forbidden(value: &Value) -> bool {
    match value {
        Value::String(s) => s.chars().any(is_forbidden),
        Value::Array(items) => items.iter().any(has_forbidden),
        Value::Object(map) => map.iter().any(|(k, v)| k.chars().any(is_forbidden) || has_forbidden(v)),
        _ => false,
    }
}

fn reread_as_yaml(value: &Value) -> Result<Value, String> {
    if has_forbidden(value) {
        return Err("yaml: control characters are not allowed".to_string());
    }
    reread_node(value, &mut 0)
}

// ---------------------------------------------------------------------------
// go-yaml v2 sorter.go: keyList.Less for string keys
// ---------------------------------------------------------------------------

fn key_less(a: &str, b: &str) -> bool {
    let ar: Vec<char> = a.chars().collect();
    let br: Vec<char> = b.chars().collect();
    let digit = |c: char| c.is_ascii_digit();
    let mut i = 0;
    while i < ar.len() && i < br.len() {
        if ar[i] == br[i] {
            i += 1;
            continue;
        }
        let al = ar[i].is_alphabetic();
        let bl = br[i].is_alphabetic();
        if al && bl {
            return ar[i] < br[i];
        }
        if al || bl {
            return bl;
        }
        let (mut an, mut bn): (i64, i64) = (0, 0);
        if ar[i] == '0' || br[i] == '0' {
            let mut j = i;
            while j > 0 && digit(ar[j - 1]) {
                if ar[j - 1] != '0' {
                    an = 1;
                    bn = 1;
                    break;
                }
                j -= 1;
            }
        }
        let mut ai = i;
        while ai < ar.len() && digit(ar[ai]) {
            an = an.wrapping_mul(10).wrapping_add(ar[ai] as i64 - '0' as i64);
            ai += 1;
        }
        let mut bi = i;
        while bi < br.len() && digit(br[bi]) {
            bn = bn.wrapping_mul(10).wrapping_add(br[bi] as i64 - '0' as i64);
            bi += 1;
        }
        if an != bn {
            return an < bn;
        }
        if ai != bi {
            return ai < bi;
        }
        return ar[i] < br[i];
    }
    ar.len() < br.len()
}

fn key_cmp(a: &str, b: &str) -> Ordering {
    if key_less(a, b) {
        Ordering::Less
    } else if key_less(b, a) {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

// ---------------------------------------------------------------------------
// go-yaml v2 resolve.go: would this plain scalar resolve to a non-string?
// ---------------------------------------------------------------------------

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// `^[-+]?(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?$`
fn is_yaml_float(s: &str) -> bool {
    let s = s.strip_prefix(['-', '+']).unwrap_or(s);
    let (mantissa, exp) = match s.find(['e', 'E']) {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    if let Some(exp) = exp {
        let exp = exp.strip_prefix(['-', '+']).unwrap_or(exp);
        if !all_digits(exp) {
            return false;
        }
    }
    if let Some(frac) = mantissa.strip_prefix('.') {
        return all_digits(frac);
    }
    match mantissa.split_once('.') {
        Some((int, frac)) => all_digits(int) && (frac.is_empty() || all_digits(frac)),
        None => all_digits(mantissa),
    }
}

/// Read 1 or 2 digits, return the number and the rest.
fn num12(s: &str) -> Option<(u32, &str)> {
    let n = s.bytes().take(2).take_while(|b| b.is_ascii_digit()).count();
    if n == 0 {
        return None;
    }
    Some((s[..n].parse().ok()?, &s[n..]))
}

/// go-yaml's `parseTimestamp` (Go `time.Parse` with its four layouts).
fn is_timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = b.iter().take_while(|c| c.is_ascii_digit()).count();
    if digits != 4 || digits == b.len() || b[4] != b'-' {
        return false;
    }
    let year: u32 = s[..4].parse().unwrap_or(0);
    let Some((month, rest)) = num12(&s[5..]) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix('-') else {
        return false;
    };
    let Some((day, rest)) = num12(rest) else { return false };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + leap as u32,
        _ => return false,
    };
    if day < 1 || day > days {
        return false;
    }
    if rest.is_empty() {
        return true;
    }
    let (with_zone, rest) = match rest.as_bytes()[0] {
        b'T' | b't' => (true, &rest[1..]),
        b' ' => (false, &rest[1..]),
        _ => return false,
    };
    let Some((hour, rest)) = num12(rest) else { return false };
    let Some(rest) = rest.strip_prefix(':') else {
        return false;
    };
    let Some((minute, rest)) = num12(rest) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix(':') else {
        return false;
    };
    let Some((second, mut rest)) = num12(rest) else {
        return false;
    };
    if hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    if let Some(frac) = rest.strip_prefix(['.', ',']) {
        let n = frac.bytes().take_while(|c| c.is_ascii_digit()).count();
        if n == 0 {
            return false;
        }
        rest = &frac[n..];
    }
    if !with_zone {
        return rest.is_empty();
    }
    if rest == "Z" {
        return true;
    }
    // ±hh:mm
    let Some(zone) = rest.strip_prefix(['+', '-']) else {
        return false;
    };
    let z = zone.as_bytes();
    z.len() == 5
        && z[2] == b':'
        && all_digits(&zone[..2])
        && all_digits(&zone[3..])
        && zone[..2].parse::<u32>().is_ok_and(|h| h < 24)
        && zone[3..].parse::<u32>().is_ok_and(|m| m < 60)
}

/// True if go-yaml's `resolve("", s)` yields `!!str`.
fn resolves_to_string(s: &str) -> bool {
    let Some(&first) = s.as_bytes().first() else {
        return false; // "" resolves to null
    };
    const MAPPED: &[&str] = &[
        "y", "Y", "yes", "Yes", "YES", "true", "True", "TRUE", "on", "On", "ON", "n", "N", "no", "No", "NO", "false",
        "False", "FALSE", "off", "Off", "OFF", "~", "null", "Null", "NULL", ".nan", ".NaN", ".NAN", ".inf", ".Inf",
        ".INF", "+.inf", "+.Inf", "+.INF", "-.inf", "-.Inf", "-.INF", "<<",
    ];
    match first {
        b'+' | b'-' | b'0'..=b'9' => {
            if MAPPED.contains(&s) || is_timestamp(s) {
                return false;
            }
            let plain = s.replace('_', "");
            if parse_int(&plain, 64).is_ok() || parse_uint(&plain, 64).is_ok() {
                return false;
            }
            !is_yaml_float(&plain)
        }
        b'.' => {
            if MAPPED.contains(&s) {
                return false;
            }
            // strconv.ParseFloat of something starting with '.'
            !(is_yaml_float(s) && s.len() > 1)
        }
        b'y' | b'Y' | b'n' | b'N' | b't' | b'T' | b'f' | b'F' | b'o' | b'O' | b'~' => !MAPPED.contains(&s),
        _ => true,
    }
}

/// `^[-+]?[0-9][0-9_]*(?::[0-5]?[0-9])+(?:\.[0-9_]*)?$`
fn is_base60_float(s: &str) -> bool {
    let Some(&c) = s.as_bytes().first() else { return false };
    if !(c == b'+' || c == b'-' || c.is_ascii_digit()) || !s.contains(':') {
        return false;
    }
    let s = s.strip_prefix(['-', '+']).unwrap_or(s);
    let (body, frac) = match s.split_once('.') {
        Some((b, f)) => (b, Some(f)),
        None => (s, None),
    };
    if let Some(f) = frac
        && !f.bytes().all(|b| b.is_ascii_digit() || b == b'_')
    {
        return false;
    }
    let mut parts = body.split(':');
    let head = parts.next().unwrap_or("");
    let hb = head.as_bytes();
    if hb.is_empty() || !hb[0].is_ascii_digit() || !hb.iter().all(|b| b.is_ascii_digit() || *b == b'_') {
        return false;
    }
    let mut count = 0;
    for part in parts {
        count += 1;
        let pb = part.as_bytes();
        let ok = match pb.len() {
            1 => pb[0].is_ascii_digit(),
            2 => (b'0'..=b'5').contains(&pb[0]) && pb[1].is_ascii_digit(),
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    count > 0
}

// ---------------------------------------------------------------------------
// libyaml emitter (yaml.v2 emitterc.go), block style only
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
}

struct Analysis {
    multiline: bool,
    block_plain_allowed: bool,
    single_quoted_allowed: bool,
    block_allowed: bool,
}

fn width(b: u8) -> usize {
    if b & 0x80 == 0 {
        1
    } else if b & 0xE0 == 0xC0 {
        2
    } else if b & 0xF0 == 0xE0 {
        3
    } else if b & 0xF8 == 0xF0 {
        4
    } else {
        1
    }
}

fn at(b: &[u8], i: usize) -> u8 {
    b.get(i).copied().unwrap_or(0)
}

fn is_printable(b: &[u8], i: usize) -> bool {
    let c = b[i];
    c == 0x0A
        || (0x20..=0x7E).contains(&c)
        || (c == 0xC2 && at(b, i + 1) >= 0xA0)
        || (c > 0xC2 && c < 0xED)
        || (c == 0xED && at(b, i + 1) < 0xA0)
        || c == 0xEE
        || (c == 0xEF
            && !(at(b, i + 1) == 0xBB && at(b, i + 2) == 0xBF)
            && !(at(b, i + 1) == 0xBF && (at(b, i + 2) == 0xBE || at(b, i + 2) == 0xBF)))
}

fn is_space(b: &[u8], i: usize) -> bool {
    at(b, i) == b' '
}

fn is_blank(b: &[u8], i: usize) -> bool {
    matches!(at(b, i), b' ' | b'\t')
}

fn is_break(b: &[u8], i: usize) -> bool {
    let c = at(b, i);
    c == b'\r'
        || c == b'\n'
        || (c == 0xC2 && at(b, i + 1) == 0x85)
        || (c == 0xE2 && at(b, i + 1) == 0x80 && (at(b, i + 2) == 0xA8 || at(b, i + 2) == 0xA9))
}

fn is_blankz(b: &[u8], i: usize) -> bool {
    is_blank(b, i) || is_break(b, i) || at(b, i) == 0
}

fn is_bom(b: &[u8]) -> bool {
    b.starts_with(&[0xEF, 0xBB, 0xBF])
}

fn analyze_scalar(value: &[u8]) -> Analysis {
    if value.is_empty() {
        return Analysis {
            multiline: false,
            block_plain_allowed: true,
            single_quoted_allowed: true,
            block_allowed: false,
        };
    }
    let mut block_indicators = false;
    let mut line_breaks = false;
    let mut special_characters = false;
    let (mut leading_space, mut leading_break, mut trailing_space, mut trailing_break) = (false, false, false, false);
    let (mut break_space, mut space_break) = (false, false);
    let (mut previous_space, mut previous_break) = (false, false);

    if value.starts_with(b"---") || value.starts_with(b"...") {
        block_indicators = true;
    }
    let mut preceded_by_whitespace = true;
    let mut i = 0;
    while i < value.len() {
        let w = width(value[i]);
        let followed_by_whitespace = i + w >= value.len() || is_blank(value, i + w);
        if i == 0 {
            match value[i] {
                b'#' | b',' | b'[' | b']' | b'{' | b'}' | b'&' | b'*' | b'!' | b'|' | b'>' | b'\'' | b'"' | b'%'
                | b'@' | b'`' => block_indicators = true,
                b'?' | b':' | b'-' if followed_by_whitespace => block_indicators = true,
                _ => {}
            }
        } else {
            match value[i] {
                b':' if followed_by_whitespace => block_indicators = true,
                b'#' if preceded_by_whitespace => block_indicators = true,
                _ => {}
            }
        }
        if !is_printable(value, i) {
            special_characters = true;
        }
        if is_space(value, i) {
            if i == 0 {
                leading_space = true;
            }
            if i + w == value.len() {
                trailing_space = true;
            }
            if previous_break {
                break_space = true;
            }
            previous_space = true;
            previous_break = false;
        } else if is_break(value, i) {
            line_breaks = true;
            if i == 0 {
                leading_break = true;
            }
            if i + w == value.len() {
                trailing_break = true;
            }
            if previous_space {
                space_break = true;
            }
            previous_space = false;
            previous_break = true;
        } else {
            previous_space = false;
            previous_break = false;
        }
        preceded_by_whitespace = is_blankz(value, i);
        i += w;
    }

    let mut a = Analysis {
        multiline: line_breaks,
        block_plain_allowed: true,
        single_quoted_allowed: true,
        block_allowed: true,
    };
    if leading_space || leading_break || trailing_space || trailing_break {
        a.block_plain_allowed = false;
    }
    if trailing_space {
        a.block_allowed = false;
    }
    if break_space {
        a.block_plain_allowed = false;
        a.single_quoted_allowed = false;
    }
    if space_break || special_characters {
        a.block_plain_allowed = false;
        a.single_quoted_allowed = false;
        a.block_allowed = false;
    }
    if line_breaks || block_indicators {
        a.block_plain_allowed = false;
    }
    a
}

struct Emitter {
    out: String,
    column: usize,
    indent: i32,
    whitespace: bool,
    indention: bool,
    mapping_context: bool,
    simple_key_context: bool,
}

impl Emitter {
    fn put(&mut self, c: char) {
        self.out.push(c);
        self.column += 1;
    }

    fn put_break(&mut self) {
        self.out.push('\n');
        self.column = 0;
    }

    /// Copy one (possibly multi-byte) character of `value` to the output.
    fn write(&mut self, value: &[u8], i: &mut usize) {
        let w = width(value[*i]);
        let end = (*i + w).min(value.len());
        self.out.push_str(&String::from_utf8_lossy(&value[*i..end]));
        self.column += 1;
        *i = end;
    }

    fn write_break(&mut self, value: &[u8], i: &mut usize) {
        if value[*i] == b'\n' {
            self.put_break();
            *i += 1;
        } else {
            self.write(value, i);
            self.column = 0;
        }
    }

    fn write_indent(&mut self) {
        let indent = self.indent.max(0) as usize;
        if !self.indention || self.column > indent || (self.column == indent && !self.whitespace) {
            self.put_break();
        }
        while self.column < indent {
            self.put(' ');
        }
        self.whitespace = true;
        self.indention = true;
    }

    fn write_indicator(&mut self, indicator: &str, need_whitespace: bool, is_whitespace: bool, is_indention: bool) {
        if need_whitespace && !self.whitespace {
            self.put(' ');
        }
        self.out.push_str(indicator);
        self.column += indicator.chars().count();
        self.whitespace = is_whitespace;
        self.indention = self.indention && is_indention;
    }

    fn emit_node(&mut self, value: &Value, mapping: bool, simple_key: bool) {
        self.mapping_context = mapping;
        self.simple_key_context = simple_key;
        match value {
            Value::Null => self.emit_scalar("null", Style::Plain),
            Value::Bool(b) => self.emit_scalar(if *b { "true" } else { "false" }, Style::Plain),
            Value::Number(n) => self.emit_scalar(&number_text(n), Style::Plain),
            Value::String(s) => self.emit_string(s),
            Value::Array(items) => {
                if items.is_empty() {
                    self.write_indicator("[", true, true, false);
                    self.write_indicator("]", false, false, false);
                    return;
                }
                let indentless = self.mapping_context && !self.indention;
                let saved = self.increase_indent(false, indentless);
                for item in items {
                    self.write_indent();
                    self.write_indicator("-", true, false, true);
                    self.emit_node(item, false, false);
                }
                self.indent = saved;
            }
            Value::Object(map) => {
                if map.is_empty() {
                    self.write_indicator("{", true, true, false);
                    self.write_indicator("}", false, false, false);
                    return;
                }
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort_by(|a, b| key_cmp(a, b));
                let saved = self.increase_indent(false, false);
                for key in keys {
                    self.write_indent();
                    let simple = key.len() <= 128 && !analyze_scalar(key.as_bytes()).multiline;
                    if simple {
                        self.mapping_context = true;
                        self.simple_key_context = true;
                        self.emit_string(key);
                        self.write_indicator(":", false, false, false);
                    } else {
                        self.write_indicator("?", true, false, true);
                        self.mapping_context = true;
                        self.simple_key_context = false;
                        self.emit_string(key);
                        self.write_indent();
                        self.write_indicator(":", true, false, true);
                    }
                    self.emit_node(&map[key], true, false);
                }
                self.indent = saved;
            }
        }
    }

    fn increase_indent(&mut self, flow: bool, indentless: bool) -> i32 {
        let saved = self.indent;
        if self.indent < 0 {
            self.indent = if flow { BEST_INDENT } else { 0 };
        } else if !indentless {
            self.indent += BEST_INDENT;
        }
        saved
    }

    /// go-yaml `encoder.stringv`: pick the requested style for a string.
    fn emit_string(&mut self, s: &str) {
        let can_use_plain = resolves_to_string(s) && !is_base60_float(s);
        let style = if s.contains('\n') {
            Style::Literal
        } else if can_use_plain {
            Style::Plain
        } else {
            Style::DoubleQuoted
        };
        self.emit_scalar(s, style);
    }

    fn emit_scalar(&mut self, s: &str, requested: Style) {
        let value = s.as_bytes();
        let a = analyze_scalar(value);
        // yaml_emitter_select_scalar_style
        let mut style = requested;
        if self.simple_key_context && a.multiline {
            style = Style::DoubleQuoted;
        }
        if style == Style::Plain && (!a.block_plain_allowed || (value.is_empty() && self.simple_key_context)) {
            style = Style::SingleQuoted;
        }
        if style == Style::SingleQuoted && !a.single_quoted_allowed {
            style = Style::DoubleQuoted;
        }
        if style == Style::Literal && (!a.block_allowed || self.simple_key_context) {
            style = Style::DoubleQuoted;
        }
        let saved = self.increase_indent(true, false);
        let allow_breaks = !self.simple_key_context;
        match style {
            Style::Plain => self.write_plain(value, allow_breaks),
            Style::SingleQuoted => self.write_single_quoted(value, allow_breaks),
            Style::DoubleQuoted => self.write_double_quoted(value, allow_breaks),
            Style::Literal => self.write_literal(value),
        }
        self.indent = saved;
    }

    fn write_plain(&mut self, value: &[u8], allow_breaks: bool) {
        if !self.whitespace {
            self.put(' ');
        }
        let mut spaces = false;
        let mut breaks = false;
        let mut i = 0;
        while i < value.len() {
            if is_space(value, i) {
                if allow_breaks && !spaces && self.column > BEST_WIDTH && !is_space(value, i + 1) {
                    self.write_indent();
                    i += 1;
                } else {
                    self.write(value, &mut i);
                }
                spaces = true;
            } else if is_break(value, i) {
                if !breaks && value[i] == b'\n' {
                    self.put_break();
                }
                self.write_break(value, &mut i);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                }
                self.write(value, &mut i);
                self.indention = false;
                spaces = false;
                breaks = false;
            }
        }
        self.whitespace = false;
        self.indention = false;
    }

    fn write_single_quoted(&mut self, value: &[u8], allow_breaks: bool) {
        self.write_indicator("'", true, false, false);
        let mut spaces = false;
        let mut breaks = false;
        let mut i = 0;
        while i < value.len() {
            if is_space(value, i) {
                if allow_breaks
                    && !spaces
                    && self.column > BEST_WIDTH
                    && i > 0
                    && i < value.len() - 1
                    && !is_space(value, i + 1)
                {
                    self.write_indent();
                    i += 1;
                } else {
                    self.write(value, &mut i);
                }
                spaces = true;
            } else if is_break(value, i) {
                if !breaks && value[i] == b'\n' {
                    self.put_break();
                }
                self.write_break(value, &mut i);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                }
                if value[i] == b'\'' {
                    self.put('\'');
                }
                self.write(value, &mut i);
                self.indention = false;
                spaces = false;
                breaks = false;
            }
        }
        self.write_indicator("'", false, false, false);
        self.whitespace = false;
        self.indention = false;
    }

    fn write_double_quoted(&mut self, value: &[u8], allow_breaks: bool) {
        let mut spaces = false;
        self.write_indicator("\"", true, false, false);
        let bom = is_bom(value);
        let mut i = 0;
        while i < value.len() {
            if !is_printable(value, i) || bom || is_break(value, i) || value[i] == b'"' || value[i] == b'\\' {
                let w = width(value[i]);
                let end = (i + w).min(value.len());
                let v = std::str::from_utf8(&value[i..end])
                    .ok()
                    .and_then(|s| s.chars().next())
                    .map_or(value[i] as u32, |c| c as u32);
                i = end;
                self.put('\\');
                match v {
                    0x00 => self.put('0'),
                    0x07 => self.put('a'),
                    0x08 => self.put('b'),
                    0x09 => self.put('t'),
                    0x0A => self.put('n'),
                    0x0B => self.put('v'),
                    0x0C => self.put('f'),
                    0x0D => self.put('r'),
                    0x1B => self.put('e'),
                    0x22 => self.put('"'),
                    0x5C => self.put('\\'),
                    0x85 => self.put('N'),
                    0xA0 => self.put('_'),
                    0x2028 => self.put('L'),
                    0x2029 => self.put('P'),
                    _ => {
                        let text = if v <= 0xFF {
                            format!("x{v:02X}")
                        } else if v <= 0xFFFF {
                            format!("u{v:04X}")
                        } else {
                            format!("U{v:08X}")
                        };
                        for c in text.chars() {
                            self.put(c);
                        }
                    }
                }
                spaces = false;
            } else if is_space(value, i) {
                if allow_breaks && !spaces && self.column > BEST_WIDTH && i > 0 && i < value.len() - 1 {
                    self.write_indent();
                    if is_space(value, i + 1) {
                        self.put('\\');
                    }
                    i += 1;
                } else {
                    self.write(value, &mut i);
                }
                spaces = true;
            } else {
                self.write(value, &mut i);
                spaces = false;
            }
        }
        self.write_indicator("\"", false, false, false);
        self.whitespace = false;
        self.indention = false;
    }

    fn write_block_scalar_hints(&mut self, value: &[u8]) {
        if is_space(value, 0) || is_break(value, 0) {
            let hint = BEST_INDENT.to_string();
            self.write_indicator(&hint, false, false, false);
        }
        let mut chomp = "";
        if value.is_empty() {
            chomp = "-";
        } else {
            let mut i = value.len() - 1;
            while value[i] & 0xC0 == 0x80 {
                i -= 1;
            }
            if !is_break(value, i) {
                chomp = "-";
            } else if i == 0 {
                chomp = "+";
            } else {
                i -= 1;
                while value[i] & 0xC0 == 0x80 {
                    i -= 1;
                }
                if is_break(value, i) {
                    chomp = "+";
                }
            }
        }
        // (libyaml's "open ended" marker is reset by this very indicator, so
        // go-yaml never emits a trailing "..." document end.)
        if !chomp.is_empty() {
            self.write_indicator(chomp, false, false, false);
        }
    }

    fn write_literal(&mut self, value: &[u8]) {
        self.write_indicator("|", true, false, false);
        self.write_block_scalar_hints(value);
        self.put_break();
        self.indention = true;
        self.whitespace = true;
        let mut breaks = true;
        let mut i = 0;
        while i < value.len() {
            if is_break(value, i) {
                self.write_break(value, &mut i);
                self.indention = true;
                breaks = true;
            } else {
                if breaks {
                    self.write_indent();
                }
                self.write(value, &mut i);
                self.indention = false;
                breaks = false;
            }
        }
    }
}

/// A JSON number as go-yaml prints it after the float64 -> JSON -> YAML trip.
fn number_text(n: &serde_json::Number) -> String {
    let f = n.as_f64().unwrap_or(0.0);
    let text = format_float_json(f);
    if !text.contains(['.', 'e']) {
        if let Ok(i) = text.parse::<i64>() {
            return i.to_string();
        }
        if let Ok(u) = text.parse::<u64>() {
            return u.to_string();
        }
    }
    match format_float_g(text.parse::<f64>().unwrap_or(f)).as_str() {
        "+Inf" => ".inf".to_string(),
        "-Inf" => "-.inf".to_string(),
        "NaN" => ".nan".to_string(),
        s => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_order_is_natural() {
        let mut keys = vec!["b", "a10", "a2", "a1", "B", "_under", "1", "list"];
        keys.sort_by(|a, b| key_cmp(a, b));
        assert_eq!(keys, ["_under", "1", "B", "a1", "a2", "a10", "b", "list"]);
    }

    #[test]
    fn scalars() {
        let v = json!({
            "at": "@home", "bool_string": "true", "colon": "key: value", "date": "2025-11-01",
            "empty": "", "f": 1.0, "e": 1e-7, "big": 12345678901234567890u64, "time": "12:30:45",
            "trail": "trailing ", "tab": "a\tb", "emoji": "☃ 😀", "num": "123", "tilde": "~",
            "nothing": null, "ver": "2025.11.1", "ip": "172.30.32.3", "q": "it's \"quoted\"",
        });
        let expected = "at: '@home'\nbig: 12345678901234567000\nbool_string: \"true\"\ncolon: 'key: value'\n\
date: \"2025-11-01\"\ne: 1e-07\nemoji: \"☃ \\U0001F600\"\nempty: \"\"\nf: 1\nip: 172.30.32.3\nnothing: null\n\
num: \"123\"\nq: it's \"quoted\"\ntab: \"a\\tb\"\ntilde: \"~\"\ntime: \"12:30:45\"\ntrail: 'trailing '\nver: 2025.11.1\n";
        assert_eq!(to_yaml(&v).unwrap(), expected);
    }

    #[test]
    fn yaml_reread_side_effects() {
        assert_eq!(
            to_yaml(&json!({"a": "x\u{7f}"})).unwrap_err(),
            "yaml: control characters are not allowed"
        );
        assert_eq!(
            to_yaml(&json!({"a": "\u{85} - &9", "b": "1e3\u{85}"})).unwrap(),
            "a: ' - &9'\nb: '1e3 '\n"
        );
        assert_eq!(fold_nel("a  \u{85} \u{85}  b c"), "a\nb c");
        assert_eq!(
            to_yaml(&json!({"a": "x\u{85}", "k\u{85}": 1})).unwrap_err(),
            "yaml: line 2: did not find expected ',' or '}'"
        );
    }

    #[test]
    fn collections() {
        let v = json!({
            "servers": ["a", "b"], "none": [], "m": {},
            "nested": {"lol": [[1, 2], [], ["x"]], "lom": [{"n": "first", "i": [1]}, {"s": "second", "i": []}]},
        });
        let expected = "m: {}\nnested:\n  lol:\n  - - 1\n    - 2\n  - []\n  - - x\n  lom:\n  - i:\n    - 1\n    \"n\": first\n  \
- i: []\n    s: second\nnone: []\nservers:\n- a\n- b\n";
        assert_eq!(to_yaml(&v).unwrap(), expected);
    }

    #[test]
    fn block_scalars_and_folding() {
        let v = json!({
            "k": "v\n", "q": "a: b\nc", "sp": "  x\n", "tr": "a\n\n", "z": 0, "crlf": "a\r\nb",
            "long": "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore",
        });
        let expected = "crlf: \"a\\r\\nb\"\nk: |\n  v\nlong: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor\n  \
incididunt ut labore\nq: |-\n  a: b\n  c\nsp: |2\n    x\ntr: |+\n  a\n\nz: 0\n";
        assert_eq!(to_yaml(&v).unwrap(), expected);
    }
}
