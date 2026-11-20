//! A small JSON reader, enough for a Competitive Companion problem.

#[derive(Clone, PartialEq, Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    /// The member `key` of an object.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Number(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_array(&self) -> &[Json] {
        match self {
            Json::Array(items) => items,
            _ => &[],
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn error<T>(&self, what: &str) -> Result<T, String> {
        Err(format!("{what} at byte {}", self.at))
    }

    fn skip_space(&mut self) {
        while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> bool {
        self.skip_space();
        let found = self.bytes.get(self.at) == Some(&byte);
        self.at += found as usize;
        found
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            Ok(value)
        } else {
            self.error("expected a value")
        }
    }

    fn string(&mut self) -> Result<String, String> {
        let mut out = Vec::new();
        loop {
            let Some(&byte) = self.bytes.get(self.at) else {
                return self.error("a string is never closed");
            };
            self.at += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    let Some(&escape) = self.bytes.get(self.at) else {
                        return self.error("a string is never closed");
                    };
                    self.at += 1;
                    let c = match escape {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'u' => {
                            let hex = self.bytes.get(self.at..self.at + 4);
                            let code = hex
                                .and_then(|h| std::str::from_utf8(h).ok())
                                .and_then(|h| u32::from_str_radix(h, 16).ok());
                            let Some(code) = code else {
                                return self.error("a `\\u` escape needs four hex digits");
                            };
                            self.at += 4;
                            char::from_u32(code).unwrap_or('\u{fffd}')
                        }
                        other => other as char,
                    };
                    out.extend(c.encode_utf8(&mut [0; 4]).as_bytes());
                }
                byte => out.push(byte),
            }
        }
        String::from_utf8(out).or_else(|_| self.error("a string is not UTF-8"))
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_space();
        match self.bytes.get(self.at) {
            Some(b'{') => {
                self.at += 1;
                let mut members = Vec::new();
                if self.eat(b'}') {
                    return Ok(Json::Object(members));
                }
                loop {
                    if !self.eat(b'"') {
                        return self.error("expected a key");
                    }
                    let key = self.string()?;
                    if !self.eat(b':') {
                        return self.error("expected `:`");
                    }
                    members.push((key, self.value()?));
                    if self.eat(b'}') {
                        return Ok(Json::Object(members));
                    }
                    if !self.eat(b',') {
                        return self.error("expected `,` or `}`");
                    }
                }
            }
            Some(b'[') => {
                self.at += 1;
                let mut items = Vec::new();
                if self.eat(b']') {
                    return Ok(Json::Array(items));
                }
                loop {
                    items.push(self.value()?);
                    if self.eat(b']') {
                        return Ok(Json::Array(items));
                    }
                    if !self.eat(b',') {
                        return self.error("expected `,` or `]`");
                    }
                }
            }
            Some(b'"') => {
                self.at += 1;
                self.string().map(Json::String)
            }
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(_) => {
                let start = self.at;
                while self
                    .bytes
                    .get(self.at)
                    .is_some_and(|b| b.is_ascii_digit() || b"+-.eE".contains(b))
                {
                    self.at += 1;
                }
                std::str::from_utf8(&self.bytes[start..self.at])
                    .ok()
                    .and_then(|text| text.parse().ok())
                    .map_or_else(|| self.error("expected a value"), |n| Ok(Json::Number(n)))
            }
            None => self.error("expected a value"),
        }
    }
}

/// Reads one JSON value.
pub fn parse(text: &str) -> Result<Json, String> {
    let mut reader = Reader {
        bytes: text.as_bytes(),
        at: 0,
    };
    let value = reader.value()?;
    reader.skip_space();
    if reader.at < reader.bytes.len() {
        return reader.error("unexpected text after the value");
    }
    Ok(value)
}
