//! Flag definitions and parsing with spf13/pflag semantics.
//!
//! Everything observable is kept identical to pflag: `--flag value`,
//! `--flag=value`, `-f value`, `-fvalue`, `-f=value`, combined shorthands,
//! `--` terminator, interspersed positional args, `NoOptDefVal` (every bool
//! flag has an implicit `NoOptDefVal` of `"true"`, so `--flag false` does NOT
//! consume `false`), the error messages and the usage rendering.

use super::complete::Completer;
use crate::gofmt::quote;

/// Value type of a flag. The names match pflag's `Value.Type()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bool,
    String,
    Int,
    Uint32,
    StringArray,
}

impl Kind {
    pub fn type_name(self) -> &'static str {
        match self {
            Kind::Bool => "bool",
            Kind::String => "string",
            Kind::Int => "int",
            Kind::Uint32 => "uint32",
            Kind::StringArray => "stringArray",
        }
    }
}

/// A flag definition (the equivalent of `pflag.Flag`).
#[derive(Clone)]
pub struct Flag {
    pub name: String,
    pub shorthand: Option<char>,
    pub usage: String,
    pub kind: Kind,
    /// Default value as text (`DefValue`), e.g. `"false"`, `""`, `"0"`, `"[]"`.
    pub def_value: String,
    /// Value used when the flag is given without `=value` (`NoOptDefVal`).
    pub no_opt_def_val: String,
    pub hidden: bool,
    pub deprecated: String,
    pub required: bool,
    pub completer: Option<Completer>,
    /// `MarkFlagFilename` extensions.
    pub filename_ext: Vec<String>,
    /// Set for the automatic `--help` flag.
    pub(crate) builtin_help: bool,
}

impl Flag {
    fn new(name: &str, kind: Kind, def_value: String, usage: &str) -> Flag {
        Flag {
            name: name.to_string(),
            shorthand: None,
            usage: usage.to_string(),
            kind,
            def_value,
            no_opt_def_val: if kind == Kind::Bool {
                "true".to_string()
            } else {
                String::new()
            },
            hidden: false,
            deprecated: String::new(),
            required: false,
            completer: None,
            filename_ext: Vec::new(),
            builtin_help: false,
        }
    }

    /// `Flags().Bool(name, default, usage)`
    pub fn bool(name: &str, default: bool, usage: &str) -> Flag {
        Flag::new(name, Kind::Bool, default.to_string(), usage)
    }

    /// `Flags().String(name, default, usage)`
    pub fn string(name: &str, default: &str, usage: &str) -> Flag {
        Flag::new(name, Kind::String, default.to_string(), usage)
    }

    /// `Flags().Int(name, default, usage)`
    pub fn int(name: &str, default: i64, usage: &str) -> Flag {
        Flag::new(name, Kind::Int, default.to_string(), usage)
    }

    /// `Flags().Uint32(name, default, usage)`
    pub fn uint32(name: &str, default: u32, usage: &str) -> Flag {
        Flag::new(name, Kind::Uint32, default.to_string(), usage)
    }

    /// `Flags().StringArray(name, []string{}, usage)`
    pub fn string_array(name: &str, usage: &str) -> Flag {
        Flag::new(name, Kind::StringArray, "[]".to_string(), usage)
    }

    /// Shorthand letter (the `P` variants in pflag, e.g. `StringP`).
    pub fn short(mut self, c: char) -> Flag {
        self.shorthand = Some(c);
        self
    }

    /// `Lookup(name).NoOptDefVal = value`
    pub fn no_opt_default(mut self, value: &str) -> Flag {
        self.no_opt_def_val = value.to_string();
        self
    }

    /// `MarkHidden`
    pub fn hidden(mut self) -> Flag {
        self.hidden = true;
        self
    }

    /// `MarkDeprecated` (also hides the flag, like pflag does).
    pub fn deprecated(mut self, message: &str) -> Flag {
        self.deprecated = message.to_string();
        self.hidden = true;
        self
    }

    /// `MarkFlagRequired`
    pub fn required(mut self) -> Flag {
        self.required = true;
        self
    }

    /// `RegisterFlagCompletionFunc`
    pub fn completion(mut self, completer: Completer) -> Flag {
        self.completer = Some(completer);
        self
    }

    /// `MarkFlagFilename(name, extensions...)`
    pub fn filename(mut self, extensions: &[&str]) -> Flag {
        self.filename_ext = extensions.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Runtime state of one flag inside a [`FlagSet`].
#[derive(Clone)]
pub struct FlagState {
    pub def: Flag,
    /// Current scalar value (canonical text); unused for arrays.
    pub value: String,
    /// Current values of a `StringArray` flag.
    pub values: Vec<String>,
    pub changed: bool,
    /// Flag comes from a parent command's persistent flags.
    pub inherited: bool,
    /// Flag was declared as a persistent flag (on this command or a parent).
    pub persistent: bool,
}

pub type NormalizeFn = fn(&str) -> String;

/// The merged, parsed flags of the command being executed.
#[derive(Clone, Default)]
pub struct FlagSet {
    pub(crate) flags: Vec<FlagState>,
    pub(crate) args: Vec<String>,
    pub(crate) normalize: Option<NormalizeFn>,
    /// Deprecation notices produced while parsing.
    pub(crate) output: String,
}

impl FlagSet {
    pub(crate) fn add(&mut self, def: &Flag, inherited: bool, persistent: bool) {
        let mut def = def.clone();
        if let Some(n) = self.normalize {
            def.name = n(&def.name);
        }
        if self.flags.iter().any(|f| f.def.name == def.name) {
            return;
        }
        let value = if def.kind == Kind::StringArray {
            String::new()
        } else {
            def.def_value.clone()
        };
        self.flags.push(FlagState {
            def,
            value,
            values: Vec::new(),
            changed: false,
            inherited,
            persistent,
        });
    }

    fn normalized(&self, name: &str) -> String {
        match self.normalize {
            Some(n) => n(name),
            None => name.to_string(),
        }
    }

    /// `Lookup(name)`
    pub fn lookup(&self, name: &str) -> Option<&FlagState> {
        let name = self.normalized(name);
        self.flags.iter().find(|f| f.def.name == name)
    }

    fn lookup_index(&self, name: &str) -> Option<usize> {
        let name = self.normalized(name);
        self.flags.iter().position(|f| f.def.name == name)
    }

    /// `ShorthandLookup(c)`
    pub fn shorthand_lookup(&self, c: char) -> Option<&FlagState> {
        self.flags.iter().find(|f| f.def.shorthand == Some(c))
    }

    /// All flags sorted by name (pflag's `VisitAll` order).
    pub fn sorted(&self) -> Vec<&FlagState> {
        let mut v: Vec<&FlagState> = self.flags.iter().collect();
        v.sort_by(|a, b| a.def.name.cmp(&b.def.name));
        v
    }

    /// Positional arguments left after parsing (`Args()`).
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// `Changed(name)`: was the flag explicitly set on the command line?
    pub fn changed(&self, name: &str) -> bool {
        self.lookup(name).is_some_and(|f| f.changed)
    }

    /// `GetString(name)` (empty string if the flag does not exist).
    pub fn get_string(&self, name: &str) -> String {
        self.lookup(name).map(|f| f.value.clone()).unwrap_or_default()
    }

    /// `GetBool(name)` (false if the flag does not exist).
    pub fn get_bool(&self, name: &str) -> bool {
        self.lookup(name).is_some_and(|f| f.value == "true")
    }

    /// `GetInt(name)` (0 if the flag does not exist).
    pub fn get_int(&self, name: &str) -> i64 {
        self.lookup(name).and_then(|f| f.value.parse().ok()).unwrap_or(0)
    }

    /// `GetUint32(name)` (0 if the flag does not exist).
    pub fn get_uint32(&self, name: &str) -> u32 {
        self.lookup(name).and_then(|f| f.value.parse().ok()).unwrap_or(0)
    }

    /// `GetStringArray(name)`
    ///
    /// pflag converts the value through its `[csv]` text form, in which a
    /// single empty string (`--flag ""`) is indistinguishable from no value:
    /// that case yields an empty list.
    pub fn get_string_array(&self, name: &str) -> Vec<String> {
        let values = self.lookup(name).map(|f| f.values.clone()).unwrap_or_default();
        if values.len() == 1 && values[0].is_empty() {
            return Vec::new();
        }
        values
    }

    /// `Set(name, value)` with pflag's validation and error text.
    fn set(&mut self, index: usize, value: &str) -> Result<(), String> {
        let flag = &mut self.flags[index];
        let parsed = match flag.def.kind {
            Kind::Bool => parse_bool(value).map(|b| b.to_string()),
            Kind::String | Kind::StringArray => Ok(value.to_string()),
            Kind::Int => parse_int(value, 64).map(|v| v.to_string()),
            Kind::Uint32 => parse_uint(value, 32).map(|v| v.to_string()),
        };
        let parsed = match parsed {
            Ok(v) => v,
            Err(cause) => {
                let flag_name = match flag.def.shorthand {
                    Some(c) => format!("-{}, --{}", c, flag.def.name),
                    None => format!("--{}", flag.def.name),
                };
                return Err(format!(
                    "invalid argument {} for {} flag: {}",
                    quote(value),
                    quote(&flag_name),
                    cause
                ));
            }
        };
        if flag.def.kind == Kind::StringArray {
            if !flag.changed {
                flag.values.clear();
            }
            flag.values.push(parsed);
        } else {
            flag.value = parsed;
        }
        flag.changed = true;
        if !flag.def.deprecated.is_empty() {
            self.output.push_str(&format!(
                "Flag --{} has been deprecated, {}\n",
                flag.def.name, flag.def.deprecated
            ));
        }
        Ok(())
    }

    /// `Parse(arguments)`.
    pub(crate) fn parse(&mut self, arguments: &[String]) -> Result<(), String> {
        self.args = Vec::with_capacity(arguments.len());
        let mut args: &[String] = arguments;
        while let Some(s) = args.first() {
            args = &args[1..];
            let b = s.as_bytes();
            if b.is_empty() || b[0] != b'-' || b.len() == 1 {
                self.args.push(s.clone());
                continue;
            }
            if b[1] == b'-' {
                if b.len() == 2 {
                    self.args.extend(args.iter().cloned());
                    break;
                }
                args = self.parse_long(s, args)?;
            } else {
                args = self.parse_short(s, args)?;
            }
        }
        Ok(())
    }

    fn parse_long<'a>(&mut self, s: &str, args: &'a [String]) -> Result<&'a [String], String> {
        let mut a = args;
        let name = &s[2..];
        if name.is_empty() || name.starts_with('-') || name.starts_with('=') {
            return Err(format!("bad flag syntax: {s}"));
        }
        let (name, inline) = match name.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (name, None),
        };
        let Some(index) = self.lookup_index(name) else {
            return Err(format!("unknown flag: --{name}"));
        };
        let value = if let Some(v) = inline {
            v.to_string()
        } else if !self.flags[index].def.no_opt_def_val.is_empty() {
            self.flags[index].def.no_opt_def_val.clone()
        } else if let Some(v) = a.first() {
            a = &a[1..];
            v.clone()
        } else {
            return Err(format!("flag needs an argument: --{name}"));
        };
        self.set(index, &value)?;
        Ok(a)
    }

    fn parse_short<'a>(&mut self, s: &str, args: &'a [String]) -> Result<&'a [String], String> {
        let mut a = args;
        let mut shorthands = &s.as_bytes()[1..];
        while !shorthands.is_empty() {
            // pflag passes the original `args` on every iteration; so do we.
            let (rest, out_args) = self.parse_single_short(shorthands, args)?;
            shorthands = rest;
            a = out_args;
        }
        Ok(a)
    }

    fn parse_single_short<'a, 'b>(
        &mut self,
        shorthands: &'b [u8],
        args: &'a [String],
    ) -> Result<(&'b [u8], &'a [String]), String> {
        let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
        let c = shorthands[0] as char;
        let mut out_shorts = &shorthands[1..];
        let mut out_args = args;
        let Some(index) = self.flags.iter().position(|f| f.def.shorthand == Some(c)) else {
            return Err(format!("unknown shorthand flag: '{}' in -{}", c, text(shorthands)));
        };
        let value;
        if shorthands.len() > 2 && shorthands[1] == b'=' {
            value = text(&shorthands[2..]);
            out_shorts = &[];
        } else if !self.flags[index].def.no_opt_def_val.is_empty() {
            value = self.flags[index].def.no_opt_def_val.clone();
        } else if shorthands.len() > 1 {
            value = text(&shorthands[1..]);
            out_shorts = &[];
        } else if let Some(v) = args.first() {
            value = v.clone();
            out_args = &args[1..];
        } else {
            return Err(format!("flag needs an argument: '{}' in -{}", c, text(shorthands)));
        }
        self.set(index, &value)?;
        Ok((out_shorts, out_args))
    }

    /// `FlagUsages()` for the flags selected by `filter`.
    pub(crate) fn usages(&self, filter: impl Fn(&FlagState) -> bool) -> String {
        let mut lines: Vec<(String, String)> = Vec::new();
        let mut maxlen = 0;
        for f in self.sorted() {
            if f.def.hidden || !filter(f) {
                continue;
            }
            let d = &f.def;
            let mut line = match d.shorthand {
                Some(c) => format!("  -{}, --{}", c, d.name),
                None => format!("      --{}", d.name),
            };
            let (varname, usage) = unquote_usage(d);
            if !varname.is_empty() {
                line.push(' ');
                line.push_str(&varname);
            }
            if !d.no_opt_def_val.is_empty() {
                match d.kind {
                    Kind::String => line.push_str(&format!("[=\"{}\"]", d.no_opt_def_val)),
                    Kind::Bool => {
                        if d.no_opt_def_val != "true" {
                            line.push_str(&format!("[={}]", d.no_opt_def_val));
                        }
                    }
                    _ => line.push_str(&format!("[={}]", d.no_opt_def_val)),
                }
            }
            // +1 accounts for the "\x00" separator pflag counts here.
            maxlen = maxlen.max(line.len() + 1);
            let mut rest = usage;
            if !default_is_zero(d) {
                if d.kind == Kind::String {
                    rest.push_str(&format!(" (default {})", quote(&d.def_value)));
                } else {
                    rest.push_str(&format!(" (default {})", d.def_value));
                }
            }
            if !d.deprecated.is_empty() {
                rest.push_str(&format!(" (DEPRECATED: {})", d.deprecated));
            }
            lines.push((line, rest));
        }
        let mut out = String::new();
        for (line, rest) in lines {
            let spacing = " ".repeat(maxlen - line.len());
            let indent = format!("\n{}", " ".repeat(maxlen + 2));
            out.push_str(&format!("{} {} {}\n", line, spacing, rest.replace('\n', &indent)));
        }
        out
    }
}

fn default_is_zero(d: &Flag) -> bool {
    match d.kind {
        Kind::Bool => d.def_value == "false" || d.def_value.is_empty(),
        Kind::Int | Kind::Uint32 => d.def_value == "0",
        Kind::String => d.def_value.is_empty(),
        Kind::StringArray => d.def_value == "[]",
    }
}

/// pflag's `UnquoteUsage`: a back-quoted word in the usage names the value.
fn unquote_usage(d: &Flag) -> (String, String) {
    let usage = &d.usage;
    if let Some(i) = usage.find('`')
        && let Some(len) = usage[i + 1..].find('`')
    {
        let name = &usage[i + 1..i + 1 + len];
        return (
            name.to_string(),
            format!("{}{}{}", &usage[..i], name, &usage[i + len + 2..]),
        );
    }
    let name = if d.kind == Kind::Bool { "" } else { d.kind.type_name() };
    (name.to_string(), usage.clone())
}

/// Go's `strconv.ParseBool`.
pub fn parse_bool(s: &str) -> Result<bool, String> {
    match s {
        "1" | "t" | "T" | "true" | "TRUE" | "True" => Ok(true),
        "0" | "f" | "F" | "false" | "FALSE" | "False" => Ok(false),
        _ => Err(format!("strconv.ParseBool: parsing {}: invalid syntax", quote(s))),
    }
}

/// Parse the magnitude of a Go integer literal (base 0 rules).
/// Returns `Err(true)` on range errors and `Err(false)` on syntax errors.
fn parse_magnitude(s: &str) -> Result<u128, bool> {
    let lower = s.to_ascii_lowercase();
    let (radix, digits, prefixed) = if let Some(r) = lower.strip_prefix("0x") {
        (16, r, true)
    } else if let Some(r) = lower.strip_prefix("0b") {
        (2, r, true)
    } else if let Some(r) = lower.strip_prefix("0o") {
        (8, r, true)
    } else if lower.len() > 1 && lower.starts_with('0') {
        (8, &lower[1..], true)
    } else {
        (10, lower.as_str(), false)
    };
    if digits.is_empty() {
        return Err(false);
    }
    // Underscores may only separate digits (or follow a base prefix).
    let bytes = digits.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'_' {
            let prev_ok = if i == 0 { prefixed } else { bytes[i - 1] != b'_' };
            let next_ok = i + 1 < bytes.len() && bytes[i + 1] != b'_';
            if !prev_ok || !next_ok {
                return Err(false);
            }
        }
    }
    let mut value: u128 = 0;
    for c in digits.chars().filter(|&c| c != '_') {
        let d = c.to_digit(radix).ok_or(false)?;
        value = value.saturating_mul(radix as u128).saturating_add(d as u128);
    }
    Ok(value)
}

/// Go's `strconv.ParseInt(s, 0, bits)`.
pub fn parse_int(s: &str, bits: u32) -> Result<i64, String> {
    let err = |range: bool| {
        format!(
            "strconv.ParseInt: parsing {}: {}",
            quote(s),
            if range { "value out of range" } else { "invalid syntax" }
        )
    };
    let (neg, rest) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let magnitude = parse_magnitude(rest).map_err(err)?;
    let limit: u128 = 1 << (bits - 1);
    if (neg && magnitude > limit) || (!neg && magnitude >= limit) {
        return Err(err(true));
    }
    Ok(if neg {
        (magnitude as i128).wrapping_neg() as i64
    } else {
        magnitude as i64
    })
}

/// Go's `strconv.ParseUint(s, 0, bits)`.
pub fn parse_uint(s: &str, bits: u32) -> Result<u64, String> {
    let err = |range: bool| {
        format!(
            "strconv.ParseUint: parsing {}: {}",
            quote(s),
            if range { "value out of range" } else { "invalid syntax" }
        )
    };
    let magnitude = parse_magnitude(s).map_err(err)?;
    if magnitude >= (1u128 << bits) {
        return Err(err(true));
    }
    Ok(magnitude as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> FlagSet {
        let mut fs = FlagSet::default();
        fs.add(&Flag::bool("follow", false, "f").short('f'), false, false);
        fs.add(&Flag::uint32("lines", 0, "n").short('n'), false, false);
        fs.add(&Flag::string("boot", "", "b").short('b'), false, false);
        fs.add(&Flag::string_array("servers", "s").short('r'), false, false);
        fs.add(&Flag::bool("fallback", true, "fb"), false, false);
        fs
    }

    fn parse(args: &[&str]) -> Result<FlagSet, String> {
        let mut fs = set();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        fs.parse(&args).map(|_| fs)
    }

    #[test]
    fn long_and_short_forms() {
        let fs = parse(&["-fn", "5", "--boot=x", "pos", "-r", "a", "-rb", "--servers", "c"]).unwrap();
        assert!(fs.get_bool("follow"));
        assert_eq!(fs.get_uint32("lines"), 5);
        assert_eq!(fs.get_string("boot"), "x");
        assert_eq!(fs.get_string_array("servers"), ["a", "b", "c"]);
        assert_eq!(fs.args(), ["pos"]);
    }

    #[test]
    fn bool_does_not_consume_next_arg() {
        let fs = parse(&["--fallback", "false"]).unwrap();
        assert!(fs.get_bool("fallback"));
        assert_eq!(fs.args(), ["false"]);
        let fs = parse(&["--fallback=false"]).unwrap();
        assert!(!fs.get_bool("fallback"));
        assert!(fs.changed("fallback"));
    }

    #[test]
    fn double_dash_terminates() {
        let fs = parse(&["a", "--", "-f", "--x"]).unwrap();
        assert!(!fs.get_bool("follow"));
        assert_eq!(fs.args(), ["a", "-f", "--x"]);
    }

    #[test]
    fn errors_match_pflag() {
        let e = |a: &[&str]| parse(a).err().unwrap();
        assert_eq!(e(&["--bogus"]), "unknown flag: --bogus");
        assert_eq!(e(&["-fx"]), "unknown shorthand flag: 'x' in -x");
        assert_eq!(e(&["--boot"]), "flag needs an argument: --boot");
        assert_eq!(e(&["-b"]), "flag needs an argument: 'b' in -b");
        assert_eq!(e(&["--=x"]), "bad flag syntax: --=x");
        assert_eq!(
            e(&["-n", "x"]),
            "invalid argument \"x\" for \"-n, --lines\" flag: strconv.ParseUint: parsing \"x\": invalid syntax"
        );
        assert_eq!(
            e(&["--fallback=maybe"]),
            "invalid argument \"maybe\" for \"--fallback\" flag: strconv.ParseBool: parsing \"maybe\": invalid syntax"
        );
        assert_eq!(
            e(&["--lines=4294967296"]),
            "invalid argument \"4294967296\" for \"-n, --lines\" flag: strconv.ParseUint: parsing \"4294967296\": value out of range"
        );
    }

    #[test]
    fn go_integer_literals() {
        assert_eq!(parse_int("0x10", 64), Ok(16));
        assert_eq!(parse_int("-010", 64), Ok(-8));
        assert_eq!(parse_int("1_000", 64), Ok(1000));
        assert_eq!(parse_int("-9223372036854775808", 64), Ok(i64::MIN));
        assert!(parse_int("9223372036854775808", 64).is_err());
        assert!(parse_int("", 64).is_err());
        assert!(parse_int("1__0", 64).is_err());
    }
}
