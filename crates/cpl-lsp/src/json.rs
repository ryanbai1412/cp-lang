//! A small JSON reader and writer, enough for the language server
//! protocol. Objects keep their key order.

use std::fmt::Write;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    /// The member `key` of an object, or `Null`.
    pub fn get(&self, key: &str) -> &Value {
        match self {
            Value::Object(members) => members
                .iter()
                .find(|(k, _)| k == key)
                .map_or(&Value::Null, |(_, v)| v),
            _ => &Value::Null,
        }
    }

    /// Follows a path of keys: `msg.at(&["params", "textDocument", "uri"])`.
    pub fn at(&self, path: &[&str]) -> &Value {
        path.iter().fold(self, |value, key| value.get(key))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Number(n) if *n >= 0.0 && *n <= f64::from(u32::MAX) => Some(*n as u32),
            _ => None,
        }
    }

    pub fn as_array(&self) -> &[Value] {
        match self {
            Value::Array(items) => items,
            _ => &[],
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
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

impl From<u32> for Value {
    fn from(n: u32) -> Value {
        Value::Number(f64::from(n))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(items: Vec<Value>) -> Value {
        Value::Array(items)
    }
}

/// Builds an object: `obj([("line", 3.into()), ("character", 0.into())])`.
pub fn obj<const N: usize>(members: [(&str, Value); N]) -> Value {
    Value::Object(
        members
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) if n.fract() == 0.0 && n.abs() < 9e15 => write!(f, "{}", *n as i64),
            Value::Number(n) if n.is_finite() => write!(f, "{n}"),
            Value::Number(_) => f.write_str("null"),
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
            Value::Object(members) => {
                f.write_char('{')?;
                for (i, (key, value)) in members.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    write_string(f, key)?;
                    write!(f, ":{value}")?;
                }
                f.write_char('}')
            }
        }
    }
}

fn write_string(f: &mut std::fmt::Formatter<'_>, s: &str) -> std::fmt::Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
            c => f.write_char(c)?,
        }
    }
    f.write_char('"')
}

/// Parses one JSON document. The error says what was wrong and where.
pub fn parse(text: &str) -> Result<Value, String> {
    let mut p = Parser {
        bytes: text.as_bytes(),
        text,
        pos: 0,
        depth: 0,
    };
    let value = p.value()?;
    p.skip_space();
    if p.pos != p.bytes.len() {
        return Err(p.fail("unexpected text after the value"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    text: &'a str,
    pos: usize,
    depth: u32,
}

impl Parser<'_> {
    fn fail(&self, what: &str) -> String {
        format!("invalid JSON at byte {}: {what}", self.pos)
    }

    fn skip_space(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn literal(&mut self, word: &str, value: Value) -> Result<Value, String> {
        if self.text[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.fail("expected a value"))
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_space();
        match self.bytes.get(self.pos) {
            Some(b'n') => self.literal("null", Value::Null),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'"') => self.string().map(Value::String),
            Some(b'[') => self.nested(|p| {
                let mut items = Vec::new();
                p.skip_space();
                if p.bytes.get(p.pos) == Some(&b']') {
                    p.pos += 1;
                    return Ok(Value::Array(items));
                }
                loop {
                    items.push(p.value()?);
                    p.skip_space();
                    if p.bytes.get(p.pos) == Some(&b']') {
                        p.pos += 1;
                        return Ok(Value::Array(items));
                    }
                    p.expect(b',')?;
                }
            }),
            Some(b'{') => self.nested(|p| {
                let mut members = Vec::new();
                p.skip_space();
                if p.bytes.get(p.pos) == Some(&b'}') {
                    p.pos += 1;
                    return Ok(Value::Object(members));
                }
                loop {
                    p.skip_space();
                    let key = p.string()?;
                    p.skip_space();
                    p.expect(b':')?;
                    members.push((key, p.value()?));
                    p.skip_space();
                    if p.bytes.get(p.pos) == Some(&b'}') {
                        p.pos += 1;
                        return Ok(Value::Object(members));
                    }
                    p.expect(b',')?;
                }
            }),
            Some(b'-' | b'0'..=b'9') => {
                let start = self.pos;
                while matches!(
                    self.bytes.get(self.pos),
                    Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                ) {
                    self.pos += 1;
                }
                self.text[start..self.pos]
                    .parse()
                    .map(Value::Number)
                    .map_err(|_| self.fail("malformed number"))
            }
            _ => Err(self.fail("expected a value")),
        }
    }

    fn nested(
        &mut self,
        body: impl FnOnce(&mut Self) -> Result<Value, String>,
    ) -> Result<Value, String> {
        if self.depth >= 128 {
            return Err(self.fail("nested too deeply"));
        }
        self.depth += 1;
        self.pos += 1;
        let value = body(self);
        self.depth -= 1;
        value
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.bytes.get(self.pos) == Some(&byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.fail(&format!("expected `{}`", byte as char)))
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let digits = self
            .text
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| self.fail("short \\u escape"))?;
        let code = u32::from_str_radix(digits, 16).map_err(|_| self.fail("bad \\u escape"))?;
        self.pos += 4;
        Ok(code)
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let start = self.pos;
            while !matches!(self.bytes.get(self.pos), Some(b'"' | b'\\') | None) {
                self.pos += 1;
            }
            out.push_str(&self.text[start..self.pos]);
            match self.bytes.get(self.pos) {
                None => return Err(self.fail("unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                _ => {
                    self.pos += 1;
                    let escape = *self
                        .bytes
                        .get(self.pos)
                        .ok_or_else(|| self.fail("unterminated string"))?;
                    self.pos += 1;
                    match escape {
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'u' => {
                            let mut code = self.hex4()?;
                            // A surrogate pair is one character.
                            if (0xd800..0xdc00).contains(&code)
                                && self.text[self.pos..].starts_with("\\u")
                            {
                                self.pos += 2;
                                let low = self.hex4()?;
                                code = 0x10000
                                    + ((code - 0xd800) << 10)
                                    + (low.wrapping_sub(0xdc00) & 0x3ff);
                            }
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        other => out.push(other as char),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_protocol_shaped_messages() {
        let text = r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/hover","params":{"position":{"line":3,"character":14},"flags":[true,false,null],"ratio":0.5}}"#;
        let value = parse(text).unwrap();
        assert_eq!(value.at(&["params", "position", "line"]).as_u32(), Some(3));
        assert_eq!(value.get("method").as_str(), Some("textDocument/hover"));
        assert!(value.get("missing").is_null());
        assert_eq!(value.to_string(), text);
    }

    #[test]
    fn strings_escape_and_unescape() {
        let value = parse(r#""a\"b\\c\n\t\u00e9\ud834\udd1e/\/""#).unwrap();
        assert_eq!(value.as_str(), Some("a\"b\\c\n\té𝄞//"));
        assert_eq!(
            Value::from("line\n\"q\"\u{1}").to_string(),
            r#""line\n\"q\"\u0001""#
        );
    }

    #[test]
    fn whitespace_and_empty_containers() {
        let value = parse(" { \"a\" : [ ] , \"b\" : { } , \"c\" : -1.5e2 } ").unwrap();
        assert_eq!(value.to_string(), r#"{"a":[],"b":{},"c":-150}"#);
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        for bad in [
            "",
            "{",
            "[1,",
            "{\"a\"}",
            "\"abc",
            "tru",
            "{\"a\":1,}",
            "1 2",
            "\"\\u12\"",
            "-",
        ] {
            assert!(parse(bad).is_err(), "{bad:?} should not parse");
        }
        let deep = "[".repeat(100_000);
        assert!(parse(&deep).is_err());
    }
}
