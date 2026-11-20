//! A small JSON value, reader and writer for the LSP benchmark client.
//! The workspace builds without network access, so there is no serde.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// Keys in insertion order.
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn obj<const N: usize>(fields: [(&str, Json); N]) -> Json {
        Json::Obj(
            fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn str(s: &str) -> Json {
        Json::Str(s.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn parse(text: &str) -> Result<Json, String> {
        let mut p = Parser {
            bytes: text.as_bytes(),
            pos: 0,
        };
        let value = p.value()?;
        p.skip_space();
        if p.pos != p.bytes.len() {
            return Err(p.error("trailing characters"));
        }
        Ok(value)
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => f.write_str("null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Num(n) => write!(f, "{n}"),
            Json::Str(s) => write_string(f, s),
            Json::Arr(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
            Json::Obj(fields) => {
                f.write_str("{")?;
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write_string(f, key)?;
                    write!(f, ":{value}")?;
                }
                f.write_str("}")
            }
        }
    }
}

fn write_string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
            c => write!(f, "{c}")?,
        }
    }
    f.write_str("\"")
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, what: &str) -> String {
        format!("JSON: {what} at byte {}", self.pos)
    }

    fn skip_space(&mut self) {
        while self
            .bytes
            .get(self.pos)
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            self.pos += 1;
        }
    }

    fn eat(&mut self, literal: &str) -> bool {
        let found = self.bytes[self.pos..].starts_with(literal.as_bytes());
        if found {
            self.pos += literal.len();
        }
        found
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_space();
        match self.bytes.get(self.pos) {
            None => Err(self.error("unexpected end")),
            Some(b'n') if self.eat("null") => Ok(Json::Null),
            Some(b't') if self.eat("true") => Ok(Json::Bool(true)),
            Some(b'f') if self.eat("false") => Ok(Json::Bool(false)),
            Some(b'"') => self.string().map(Json::Str),
            Some(b'[') => {
                self.pos += 1;
                let mut items = vec![];
                loop {
                    self.skip_space();
                    if self.eat("]") {
                        return Ok(Json::Arr(items));
                    }
                    if !items.is_empty() && !self.eat(",") {
                        return Err(self.error("expected `,` or `]`"));
                    }
                    items.push(self.value()?);
                }
            }
            Some(b'{') => {
                self.pos += 1;
                let mut fields = vec![];
                loop {
                    self.skip_space();
                    if self.eat("}") {
                        return Ok(Json::Obj(fields));
                    }
                    if !fields.is_empty() && !self.eat(",") {
                        return Err(self.error("expected `,` or `}`"));
                    }
                    self.skip_space();
                    let key = self.string()?;
                    self.skip_space();
                    if !self.eat(":") {
                        return Err(self.error("expected `:`"));
                    }
                    fields.push((key, self.value()?));
                }
            }
            Some(_) => {
                let start = self.pos;
                while self
                    .bytes
                    .get(self.pos)
                    .is_some_and(|b| b.is_ascii_digit() || b"+-.eE".contains(b))
                {
                    self.pos += 1;
                }
                std::str::from_utf8(&self.bytes[start..self.pos])
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .map(Json::Num)
                    .ok_or_else(|| self.error("expected a value"))
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        if !self.eat("\"") {
            return Err(self.error("expected a string"));
        }
        let mut out = Vec::new();
        loop {
            let b = *self
                .bytes
                .get(self.pos)
                .ok_or_else(|| self.error("unterminated string"))?;
            self.pos += 1;
            match b {
                b'"' => return String::from_utf8(out).map_err(|_| self.error("invalid UTF-8")),
                b'\\' => {
                    let esc = *self
                        .bytes
                        .get(self.pos)
                        .ok_or_else(|| self.error("unterminated escape"))?;
                    self.pos += 1;
                    let c = match esc {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'u' => self.unicode_escape()?,
                        b'"' | b'\\' | b'/' => esc as char,
                        _ => return Err(self.error("unknown escape")),
                    };
                    out.extend(c.encode_utf8(&mut [0; 4]).as_bytes());
                }
                b => out.push(b),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let digits = self
            .bytes
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| self.error("short \\u escape"))?;
        let value = std::str::from_utf8(digits)
            .ok()
            .and_then(|s| u32::from_str_radix(s, 16).ok())
            .ok_or_else(|| self.error("bad \\u escape"))?;
        self.pos += 4;
        Ok(value)
    }

    fn unicode_escape(&mut self) -> Result<char, String> {
        let high = self.hex4()?;
        let code = if (0xd800..0xdc00).contains(&high) && self.eat("\\u") {
            let low = self.hex4()?;
            0x10000 + ((high - 0xd800) << 10) + (low.wrapping_sub(0xdc00) & 0x3ff)
        } else {
            high
        };
        Ok(char::from_u32(code).unwrap_or('\u{fffd}'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_message() {
        let text = r#"{"jsonrpc":"2.0","id":3,"result":{"contents":"a \"b\"\n\tc","range":[1,2.5,-3],"ok":true,"none":null}}"#;
        let value = Json::parse(text).unwrap();
        assert_eq!(value.get("id").and_then(Json::as_f64), Some(3.0));
        assert_eq!(
            value
                .get("result")
                .and_then(|r| r.get("contents"))
                .and_then(Json::as_str),
            Some("a \"b\"\n\tc")
        );
        assert_eq!(value.to_string(), text);
    }

    #[test]
    fn reads_escapes_and_whitespace() {
        let value = Json::parse(" { \"k\" : [ \"\\u00e9\\ud83d\\ude00\\/\" , 1e2 ] } ").unwrap();
        assert_eq!(
            value,
            Json::obj([("k", Json::Arr(vec![Json::str("é😀/"), Json::Num(100.0)]))])
        );
    }

    #[test]
    fn rejects_malformed_input() {
        for bad in ["", "{", "[1,]", "{\"a\" 1}", "\"abc", "nul", "1 2"] {
            assert!(Json::parse(bad).is_err(), "{bad}");
        }
    }
}
