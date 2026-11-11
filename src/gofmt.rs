//! Small helpers reproducing Go's text formatting where it is observable in
//! the CLI output (`%q`, `%v` of slices, float formatting, JSON encoding).

use serde_json::Value;

/// Go's `strconv.Quote` / `%q` for strings.
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{07}' => out.push_str("\\a"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0b}' => out.push_str("\\v"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Go's `%v` of a `[]string`: `[a b c]`.
pub fn list(items: &[String]) -> String {
    format!("[{}]", items.join(" "))
}

/// Shortest decimal digits and decimal exponent of a finite float, such that
/// `value = 0.d1d2d3... * 10^exp10` (Go's `decimalSlice{d, dp}`).
fn shortest_digits(f: f64) -> (String, i32) {
    if f == 0.0 {
        return ("0".to_string(), 1);
    }
    let s = format!("{:e}", f.abs());
    let (mantissa, exp) = s.split_once('e').unwrap_or((&s, "0"));
    let digits: String = mantissa.chars().filter(|c| c.is_ascii_digit()).collect();
    let exp: i32 = exp.parse().unwrap_or(0);
    (digits, exp + 1)
}

/// `%e` with shortest precision: `d.ddde±XX`.
fn format_e(neg: bool, digits: &str, dp: i32) -> String {
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&digits[..1]);
    if digits.len() > 1 {
        out.push('.');
        out.push_str(&digits[1..]);
    }
    let exp = dp - 1;
    out.push('e');
    out.push(if exp < 0 { '-' } else { '+' });
    out.push_str(&format!("{:02}", exp.abs()));
    out
}

/// `%f` with shortest precision.
fn format_f(neg: bool, digits: &str, dp: i32) -> String {
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    if dp <= 0 {
        out.push_str("0.");
        out.push_str(&"0".repeat((-dp) as usize));
        out.push_str(digits);
    } else if dp as usize >= digits.len() {
        out.push_str(digits);
        out.push_str(&"0".repeat(dp as usize - digits.len()));
    } else {
        out.push_str(&digits[..dp as usize]);
        out.push('.');
        out.push_str(&digits[dp as usize..]);
    }
    out
}

/// Go's `strconv.FormatFloat(f, 'g', -1, 64)` (used by go-yaml for floats).
pub fn format_float_g(f: f64) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 { "+Inf" } else { "-Inf" }.to_string();
    }
    let neg = f.is_sign_negative();
    let (digits, dp) = shortest_digits(f);
    let exp = dp - 1;
    if !(-4..6).contains(&exp) {
        format_e(neg, &digits, dp)
    } else {
        format_f(neg, &digits, dp)
    }
}

/// How Go's `encoding/json` writes a float64.
pub fn format_float_json(f: f64) -> String {
    let neg = f.is_sign_negative();
    let abs = f.abs();
    let (digits, dp) = shortest_digits(f);
    if abs != 0.0 && !(1e-6..1e21).contains(&abs) {
        let mut s = format_e(neg, &digits, dp);
        // clean up e-09 to e-9
        let n = s.len();
        if n >= 4 && s.as_bytes()[n - 4] == b'e' && s.as_bytes()[n - 3] == b'-' && s.as_bytes()[n - 2] == b'0' {
            s.remove(n - 2);
        }
        s
    } else {
        format_f(neg, &digits, dp)
    }
}

fn json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '<' | '>' | '&' | '\u{2028}' | '\u{2029}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn json_value(out: &mut String, v: &Value) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.push_str(&i.to_string());
            } else if let Some(u) = n.as_u64() {
                out.push_str(&u.to_string());
            } else {
                out.push_str(&format_float_json(n.as_f64().unwrap_or(0.0)));
            }
        }
        Value::String(s) => json_string(out, s),
        Value::Array(a) => {
            out.push('[');
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                json_value(out, item);
            }
            out.push(']');
        }
        Value::Object(m) => {
            // serde_json's map (without `preserve_order`) is sorted by key,
            // exactly like Go's json.Marshal of a map.
            out.push('{');
            for (i, (k, item)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                json_string(out, k);
                out.push(':');
                json_value(out, item);
            }
            out.push('}');
        }
    }
}

/// Encode like Go's `json.Marshal`: compact, keys sorted, `<`, `>`, `&`
/// escaped as `\u00XX`, Go float formatting. Used for request bodies so they
/// are byte-identical to what the Go CLI sends.
pub fn json_marshal(v: &Value) -> String {
    let mut out = String::new();
    json_value(&mut out, v);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn float_g() {
        assert_eq!(format_float_g(0.25), "0.25");
        assert_eq!(format_float_g(1e21), "1e+21");
        assert_eq!(format_float_g(1e20), "1e+20");
        assert_eq!(format_float_g(1234567.5), "1.2345675e+06");
        assert_eq!(format_float_g(100000.5), "100000.5");
        assert_eq!(format_float_g(1e-7), "1e-07");
        assert_eq!(format_float_g(0.0001), "0.0001");
        assert_eq!(format_float_g(0.00001), "1e-05");
        assert_eq!(format_float_g(-1.5), "-1.5");
        assert_eq!(format_float_g(1.8446744073709552e19), "1.8446744073709552e+19");
    }

    #[test]
    fn float_json() {
        assert_eq!(format_float_json(1.0), "1");
        assert_eq!(format_float_json(1e21), "1e+21");
        assert_eq!(format_float_json(1e-7), "1e-7");
        assert_eq!(format_float_json(12345678901234567890.0), "12345678901234567000");
        assert_eq!(format_float_json(-0.0), "-0");
        assert_eq!(format_float_json(0.000001), "0.000001");
    }

    #[test]
    fn marshal_like_go() {
        let v = json!({"servers": ["a", "b<c>&"], "fallback": false, "n": 5, "f": 0.5});
        assert_eq!(
            json_marshal(&v),
            r#"{"f":0.5,"fallback":false,"n":5,"servers":["a","b\u003cc\u003e\u0026"]}"#
        );
    }

    #[test]
    fn quote_like_go() {
        assert_eq!(quote("a\"b\\c\n"), r#""a\"b\\c\n""#);
        assert_eq!(list(&["a".into(), "b".into()]), "[a b]");
    }
}
