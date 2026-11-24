//! Zero-dependency JSON value, RFC 8259 parser, and deterministic writer.
//!
//! Objects use `BTreeMap<String, Value>` so serialization and equality are
//! deterministic regardless of key insertion order and duplicate keys resolve
//! deterministically (last wins) in O(log n) lookup time.

use std::collections::BTreeMap;
use std::fmt::{self, Write};

mod writer;
pub use writer::{write_with, Number, Writer};

const MAX_DEPTH: usize = 128;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for Error {}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Follows a path of object keys, returning `&Value::Null` if any step is missing.
    pub fn at(&self, path: &[&str]) -> &Value {
        path.iter()
            .fold(self, |val, key| val.get(key).unwrap_or(&Value::Null))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f)
                if f.is_finite()
                    && f.fract() == 0.0
                    && *f >= i64::MIN as f64
                    && *f < 9223372036854775808.0 =>
            {
                Some(*f as i64)
            }
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Int(n) if *n >= 0 => Some(*n as u64),
            Value::Float(f)
                if f.is_finite()
                    && *f >= 0.0
                    && f.fract() == 0.0
                    && *f < 18446744073709551616.0 =>
            {
                Some(*f as u64)
            }
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|n| u32::try_from(n).ok())
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|n| usize::try_from(n).ok())
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(n) => Some(*n as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::String(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Value {
        Value::Int(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Value {
        Value::Int(i64::from(n))
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Value {
        Value::Int(i64::from(n))
    }
}

impl From<usize> for Value {
    fn from(n: usize) -> Value {
        match i64::try_from(n) {
            Ok(i) => Value::Int(i),
            Err(_) => Value::Float(n as f64),
        }
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Value {
        Value::Float(f)
    }
}

impl From<Vec<Value>> for Value {
    fn from(items: Vec<Value>) -> Value {
        Value::Array(items)
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(map: BTreeMap<String, Value>) -> Value {
        Value::Object(map)
    }
}

/// Builds a `Value::Object` from an array of `(key, value)` pairs.
/// Inlined into a call of one non-generic builder, so each array length
/// costs a few bytes of wasm rather than a copy of the map building.
#[inline]
pub fn obj<const N: usize>(members: [(&str, Value); N]) -> Value {
    object(&mut members.into_iter())
}

fn object(members: &mut dyn Iterator<Item = (&str, Value)>) -> Value {
    let mut map = BTreeMap::new();
    for (k, v) in members {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

/// Writes `s` as a quoted, RFC 8259-escaped JSON string, safe to embed
/// inside an HTML `<script>` tag too: `<`, U+2028 and U+2029 are escaped.
/// The one escaper of every JSON the toolchain writes.
pub fn write_string(out: &mut dyn Write, s: &str) -> fmt::Result {
    out.write_char('"')?;
    for c in s.chars() {
        match c {
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            '\x08' => out.write_str("\\b")?,
            '\x0c' => out.write_str("\\f")?,
            '\n' => out.write_str("\\n")?,
            '\r' => out.write_str("\\r")?,
            '\t' => out.write_str("\\t")?,
            '<' => out.write_str("\\u003c")?,
            '\u{2028}' => out.write_str("\\u2028")?,
            '\u{2029}' => out.write_str("\\u2029")?,
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32)?,
            c => out.write_char(c)?,
        }
    }
    out.write_char('"')
}

/// Returns `s` as a quoted, escaped JSON string ([`write_string`]).
pub fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    let _ = write_string(&mut out, s);
    out
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) if n.is_finite() => write!(f, "{n}"),
            Value::Float(_) => f.write_str("null"),
            Value::String(s) => write_string(f, s),
            Value::Array(items) => {
                f.write_char('[')?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_char(']')
            }
            Value::Object(map) => {
                f.write_char('{')?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    write_string(f, k)?;
                    f.write_char(':')?;
                    write!(f, "{v}")?;
                }
                f.write_char('}')
            }
        }
    }
}

pub fn to_string(val: &Value) -> String {
    val.to_string()
}

/// Where an object's keys go when a published shape fixes their order: the
/// object at each dotted `path` (`""` the top, `"a.b"` under `a` then `b`,
/// an array's items at the array's path) writes the `keys` it has first, in
/// that order, then the rest sorted.
pub struct KeyOrder<'a>(pub &'a [(&'a str, &'a [&'a str])]);

/// [`to_string`] with the object keys in `order`.
pub fn to_string_ordered(val: &Value, order: &KeyOrder) -> String {
    let mut out = String::new();
    write_ordered(val, "", order, &mut out);
    out
}

fn write_ordered(val: &Value, path: &str, order: &KeyOrder, out: &mut String) {
    match val {
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_ordered(item, path, order, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let first = (order.0.iter())
                .find(|(at, _)| *at == path)
                .map_or(&[][..], |(_, keys)| *keys);
            let listed = first.iter().filter_map(|k| map.get_key_value(*k));
            let rest = map.iter().filter(|(k, _)| !first.contains(&k.as_str()));
            out.push('{');
            for (i, (k, v)) in listed.chain(rest).enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let _ = write_string(out, k);
                out.push(':');
                let inner = match path {
                    "" => k.clone(),
                    _ => format!("{path}.{k}"),
                };
                write_ordered(v, &inner, order, out);
            }
            out.push('}');
        }
        other => {
            let _ = write!(out, "{other}");
        }
    }
}

pub fn to_string_pretty(val: &Value) -> String {
    let mut out = String::new();
    write_pretty(val, 0, &mut out);
    out.push('\n');
    out
}

fn write_pretty(val: &Value, indent: usize, out: &mut String) {
    match val {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(n) => {
            let _ = write!(out, "{n}");
        }
        Value::Float(n) => {
            if n.is_finite() {
                let _ = write!(out, "{n}");
            } else {
                out.push_str("null");
            }
        }
        Value::String(s) => {
            let _ = write_string(out, s);
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                push_indent(indent + 2, out);
                write_pretty(item, indent + 2, out);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(indent, out);
            out.push(']');
        }
        Value::Object(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            let len = map.len();
            for (i, (k, v)) in map.iter().enumerate() {
                push_indent(indent + 2, out);
                let _ = write_string(out, k);
                out.push_str(": ");
                write_pretty(v, indent + 2, out);
                if i + 1 < len {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(indent, out);
            out.push('}');
        }
    }
}

fn push_indent(n: usize, out: &mut String) {
    for _ in 0..n {
        out.push(' ');
    }
}

pub fn parse(input: &str) -> Result<Value, Error> {
    let mut p = Parser {
        bytes: input.as_bytes(),
        pos: 0,
        depth: 0,
    };
    p.skip_ws();
    let val = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.bytes.len() {
        return Err(p.err("trailing data"));
    }
    Ok(val)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn err(&self, msg: impl Into<String>) -> Error {
        Error {
            offset: self.pos,
            message: msg.into(),
        }
    }

    fn err_at(&self, offset: usize, msg: impl Into<String>) -> Error {
        Error {
            offset,
            message: msg.into(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<Value, Error> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => self.parse_literal(b"null", Value::Null),
            Some(b't') => self.parse_literal(b"true", Value::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", Value::Bool(false)),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(self.err("unexpected byte")),
            None => Err(self.err("unexpected end of input")),
        }
    }

    fn parse_literal(&mut self, word: &[u8], val: Value) -> Result<Value, Error> {
        if self.bytes[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(val)
        } else {
            Err(self.err("invalid literal"))
        }
    }

    fn parse_array(&mut self) -> Result<Value, Error> {
        if self.depth >= MAX_DEPTH {
            return Err(self.err("nesting depth limit exceeded"));
        }
        self.depth += 1;
        self.pos += 1;
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(Value::Array(out));
        }
        loop {
            out.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    self.depth -= 1;
                    return Ok(Value::Array(out));
                }
                _ => return Err(self.err("expected `,` or `]`")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Value, Error> {
        if self.depth >= MAX_DEPTH {
            return Err(self.err("nesting depth limit exceeded"));
        }
        self.depth += 1;
        self.pos += 1;
        let mut map = BTreeMap::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(Value::Object(map));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("expected string key"));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.err("expected `:`"));
            }
            self.pos += 1;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    self.depth -= 1;
                    return Ok(Value::Object(map));
                }
                _ => return Err(self.err("expected `,` or `}`")),
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, Error> {
        let start = self.pos;
        let Some(slice) = self.bytes.get(self.pos..self.pos + 4) else {
            return Err(self.err_at(start, "incomplete `\\u` escape"));
        };
        let s =
            std::str::from_utf8(slice).map_err(|_| self.err_at(start, "invalid `\\u` escape"))?;
        let v =
            u16::from_str_radix(s, 16).map_err(|_| self.err_at(start, "invalid `\\u` escape"))?;
        self.pos += 4;
        Ok(v)
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        let open = self.pos;
        self.pos += 1;
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let esc_pos = self.pos;
                    let Some(&esc) = self.bytes.get(self.pos) else {
                        return Err(self.err("unterminated escape"));
                    };
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\x08'),
                        b'f' => out.push('\x0c'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex4()?;
                            if (0xD800..=0xDBFF).contains(&code) {
                                if !self.bytes[self.pos..].starts_with(b"\\u") {
                                    return Err(self.err_at(esc_pos, "unpaired high surrogate"));
                                }
                                self.pos += 2;
                                let low = self.parse_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(self.err_at(esc_pos, "invalid low surrogate"));
                                }
                                let cp = 0x10000
                                    + (((u32::from(code) - 0xD800) << 10)
                                        | (u32::from(low) - 0xDC00));
                                let ch = char::from_u32(cp).ok_or_else(|| {
                                    self.err_at(esc_pos, "invalid unicode scalar")
                                })?;
                                out.push(ch);
                            } else if (0xDC00..=0xDFFF).contains(&code) {
                                return Err(self.err_at(esc_pos, "unpaired low surrogate"));
                            } else {
                                let ch = char::from_u32(u32::from(code)).ok_or_else(|| {
                                    self.err_at(esc_pos, "invalid unicode scalar")
                                })?;
                                out.push(ch);
                            }
                        }
                        _ => return Err(self.err_at(esc_pos, "invalid escape sequence")),
                    }
                }
                0x00..=0x1F => return Err(self.err("unescaped control character in string")),
                _ => {
                    let start = self.pos;
                    while self.pos < self.bytes.len()
                        && self.bytes[self.pos] != b'"'
                        && self.bytes[self.pos] != b'\\'
                        && self.bytes[self.pos] >= 0x20
                    {
                        self.pos += 1;
                    }
                    let chunk = std::str::from_utf8(&self.bytes[start..self.pos])
                        .map_err(|_| self.err_at(start, "invalid UTF-8"))?;
                    out.push_str(chunk);
                }
            }
        }
        Err(self.err_at(open, "unterminated string"))
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let start = self.pos;
        let mut is_float = false;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.err("leading zero in number"));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.err("expected digit")),
        }
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            let frac_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if self.pos == frac_start {
                return Err(self.err("expected digit after decimal point"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if self.pos == exp_start {
                return Err(self.err("expected digit in exponent"));
            }
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.err_at(start, "invalid number"))?;
        if !is_float {
            if let Ok(i) = s.parse::<i64>() {
                return Ok(Value::Int(i));
            }
        }
        let f = s
            .parse::<f64>()
            .map_err(|_| self.err_at(start, "invalid number"))?;
        if !f.is_finite() {
            return Err(self.err_at(start, "non-finite number"));
        }
        Ok(Value::Float(f))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_key_order_puts_listed_keys_first_at_their_path_and_sorts_the_rest() {
        let val = parse(r#"{"z":1,"b":{"y":[{"q":1,"p":2}],"x":2},"a":3}"#).unwrap();
        let order = KeyOrder(&[("", &["z", "b"]), ("b", &["y"]), ("b.y", &["q"])]);
        assert_eq!(
            to_string_ordered(&val, &order),
            r#"{"z":1,"b":{"y":[{"q":1,"p":2}],"x":2},"a":3}"#
        );
        assert_eq!(to_string_ordered(&val, &KeyOrder(&[])), to_string(&val));
    }

    use super::*;

    #[test]
    fn parses_and_formats_escapes_and_surrogates() {
        let raw = r#""a\"b\\c\/d\be\ff\ng\rh\ti\u0041\u0001\uD83D\uDE00""#;
        let v = parse(raw).unwrap();
        assert_eq!(
            v.as_str(),
            Some("a\"b\\c/d\x08e\x0cf\ng\rh\tiA\u{0001}\u{1F600}")
        );
        let serialized = to_string(&v);
        assert_eq!(parse(&serialized).unwrap(), v);
        assert_eq!(escape_string("a\tb\u{1}\x08\x0c"), "\"a\\tb\\u0001\\b\\f\"");
    }

    #[test]
    fn rejects_lone_surrogates_and_unescaped_controls() {
        for bad in [
            r#""\uD800""#,
            r#""\uDC00""#,
            r#""\uD800abc""#,
            r#""\uD800\u0041""#,
            "\"a\nb\"",
            "\"a\tb\"",
            r#""\x41""#,
        ] {
            assert!(parse(bad).is_err(), "expected error for {bad:?}");
        }
    }

    #[test]
    fn parses_numbers_including_i64_extremes() {
        assert_eq!(parse("0").unwrap(), Value::Int(0));
        assert_eq!(parse("-0").unwrap(), Value::Int(0));
        assert_eq!(parse("9223372036854775807").unwrap(), Value::Int(i64::MAX));
        assert_eq!(parse("-9223372036854775808").unwrap(), Value::Int(i64::MIN));
        assert_eq!(
            parse("9223372036854775807").unwrap().as_i64(),
            Some(i64::MAX)
        );
        assert_eq!(
            parse("-9223372036854775808").unwrap().as_i64(),
            Some(i64::MIN)
        );
        assert_eq!(to_string(&Value::Int(i64::MAX)), "9223372036854775807");
        assert_eq!(to_string(&Value::Int(i64::MIN)), "-9223372036854775808");
        assert!(matches!(
            parse("9223372036854775808").unwrap(),
            Value::Float(_)
        ));
        assert_eq!(parse("1.5e2").unwrap(), Value::Float(150.0));
        assert_eq!(parse("-3.25").unwrap(), Value::Float(-3.25));
    }

    #[test]
    fn rejects_malformed_numbers_and_reports_byte_offsets() {
        for (bad, expected_offset) in [
            ("01", 1),
            ("-01", 2),
            ("1.", 2),
            ("1e", 2),
            ("1e+", 3),
            ("[1, 2,]", 6),
            ("{\"a\": 1,}", 8),
            ("true false", 5),
            ("", 0),
        ] {
            let err = parse(bad).unwrap_err();
            assert_eq!(
                err.offset, expected_offset,
                "wrong offset for {bad:?}: {err}"
            );
        }
    }

    #[test]
    fn handles_deep_nesting_and_enforces_limit() {
        let ok = format!("{}1{}", "[".repeat(64), "]".repeat(64));
        assert!(parse(&ok).is_ok());

        let too_deep = format!(
            "{}1{}",
            "[".repeat(MAX_DEPTH + 1),
            "]".repeat(MAX_DEPTH + 1)
        );
        let err = parse(&too_deep).unwrap_err();
        assert!(err.message.contains("depth"), "unexpected error: {err}");
    }

    #[test]
    fn deterministic_sorted_keys_and_pretty_roundtrip() {
        let v = obj([
            ("z", 1.into()),
            ("a", obj([("k", "v".into()), ("b", true.into())])),
            ("m", Value::Array(vec![Value::Null, Value::Int(-7)])),
        ]);
        let compact = to_string(&v);
        assert_eq!(compact, r#"{"a":{"b":true,"k":"v"},"m":[null,-7],"z":1}"#);
        let pretty = to_string_pretty(&v);
        assert_eq!(parse(&pretty).unwrap(), v);
        assert_eq!(v.at(&["a", "k"]).as_str(), Some("v"));
        assert!(v.at(&["a", "missing"]).is_null());
    }

    #[test]
    fn script_safe_escaping_escapes_script_breakers_and_roundtrips() {
        let s = "</script>\u{2028}\u{2029}\n\t\"\\";
        let escaped = escape_string(s);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('\u{2028}'));
        assert!(!escaped.contains('\u{2029}'));
        assert_eq!(parse(&escaped).unwrap(), Value::String(s.to_string()));
    }
}
