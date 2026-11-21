//! The program's input: one buffer, read token by token.

/// Why a read failed, with where in the input.
pub(crate) struct InputError {
    pub message: String,
}

/// Gives the next bytes of an input that arrives while the program runs,
/// and `None` once there are no more: what a judge sends a solution.
pub type More<'a> = Box<dyn FnMut() -> Option<Vec<u8>> + Send + 'a>;

pub(crate) struct Input<'a> {
    data: std::borrow::Cow<'a, [u8]>,
    pos: usize,
    /// Where the bytes after `data` come from, until it runs dry.
    more: Option<More<'a>>,
    /// What the bytes are, for messages: `input`, or the `solution's
    /// output` a judge reads with `read_output()`.
    name: &'static str,
}

impl<'a> Input<'a> {
    pub fn new(data: &'a [u8]) -> Input<'a> {
        Input {
            data: std::borrow::Cow::Borrowed(data),
            pos: 0,
            more: None,
            name: "input",
        }
    }

    /// An input that starts empty and is fed by `more`.
    pub fn live(more: More<'a>) -> Input<'a> {
        Input {
            data: std::borrow::Cow::Owned(Vec::new()),
            pos: 0,
            more: Some(more),
            name: "input",
        }
    }

    /// The same stream, called `name` in messages.
    pub fn named(self, name: &'static str) -> Input<'a> {
        Input { name, ..self }
    }

    /// The byte at `pos`, waiting for it when the input is live.
    fn byte(&mut self, pos: usize) -> Option<u8> {
        while pos >= self.data.len() {
            let chunk = self.more.as_mut().and_then(|more| more());
            match chunk {
                Some(chunk) => self.data.to_mut().extend_from_slice(&chunk),
                None => {
                    self.more = None;
                    return None;
                }
            }
        }
        Some(self.data[pos])
    }

    /// True when the next read has to wait for bytes that have not
    /// arrived: whoever sends them may be waiting for what was printed.
    pub fn starved(&self) -> bool {
        self.more.is_some()
            && self.data[self.pos.min(self.data.len())..]
                .iter()
                .all(u8::is_ascii_whitespace)
    }

    /// True when only blanks are left to read.
    pub fn at_eof(&mut self) -> bool {
        self.skip_space();
        self.byte(self.pos).is_none()
    }

    fn skip_space(&mut self) {
        while self.byte(self.pos).is_some_and(|b| b.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    /// Line and column of a byte of the input, both from 1.
    fn at(&self, pos: usize) -> String {
        let before = &self.data[..pos.min(self.data.len())];
        let line = before.iter().filter(|b| **b == b'\n').count() + 1;
        let col = before.iter().rev().take_while(|b| **b != b'\n').count() + 1;
        match self.name {
            "input" => format!("input line {line}, column {col}"),
            name => format!("line {line}, column {col} of the {name}"),
        }
    }

    /// Goes back to `start`, to read what is there another way.
    pub fn rewind(&mut self, start: usize) {
        self.pos = start;
    }

    pub fn fail(&self, start: usize, message: String) -> InputError {
        InputError {
            message: format!("{message} ({})", self.at(start)),
        }
    }

    /// The next run of non-space bytes, and where it starts.
    pub fn token(&mut self, what: &str) -> Result<(&[u8], usize), InputError> {
        self.skip_space();
        let start = self.pos;
        while self
            .byte(self.pos)
            .is_some_and(|b| !b.is_ascii_whitespace())
        {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(self.fail(
                start,
                format!("the {} ended where {what} was expected", self.name),
            ));
        }
        Ok((&self.data[start..self.pos], start))
    }

    pub fn int(&mut self) -> Result<(i64, usize), InputError> {
        let (token, start) = self.token("a number")?;
        let (negative, digits) = match token {
            [b'-', rest @ ..] => (true, rest),
            [b'+', rest @ ..] => (false, rest),
            rest => (false, rest),
        };
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            let message = format!("expected a number, found `{}`", lossy(token));
            return Err(self.fail(start, message));
        }
        let mut value: Option<i64> = Some(0);
        for digit in digits {
            let d = i64::from(digit - b'0');
            value = value.and_then(|v| v.checked_mul(10)).and_then(|v| {
                if negative {
                    v.checked_sub(d)
                } else {
                    v.checked_add(d)
                }
            });
        }
        match value.filter(|v| *v != i64::MIN) {
            Some(value) => Ok((value, start)),
            None => {
                let message = format!("`{}` does not fit in an `Int`", lossy(token));
                Err(self.fail(start, message))
            }
        }
    }

    /// A whole number of any size that `T` holds: an `I128` or a `U64`.
    pub fn wide<T: std::str::FromStr>(&mut self, name: &str) -> Result<T, InputError> {
        let (token, start) = self.token("a number")?;
        let digits = token
            .strip_prefix(b"-")
            .or(token.strip_prefix(b"+"))
            .unwrap_or(token);
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            let message = format!("expected a number, found `{}`", lossy(token));
            return Err(self.fail(start, message));
        }
        let parsed = std::str::from_utf8(token)
            .ok()
            .and_then(|text| text.parse().ok());
        match parsed {
            Some(value) => Ok(value),
            None => {
                let message = format!("`{}` does not fit in {name}", lossy(token));
                Err(self.fail(start, message))
            }
        }
    }

    pub fn float(&mut self) -> Result<f64, InputError> {
        let (token, start) = self.token("a number")?;
        let parsed = std::str::from_utf8(token)
            .ok()
            .and_then(|text| text.parse::<f64>().ok());
        match parsed {
            Some(value) => Ok(value),
            None => {
                let message = format!("expected a number, found `{}`", lossy(token));
                Err(self.fail(start, message))
            }
        }
    }

    /// The next byte that is not white space.
    pub fn char(&mut self) -> Result<u8, InputError> {
        self.skip_space();
        match self.byte(self.pos) {
            Some(byte) => {
                self.pos += 1;
                Ok(byte)
            }
            None => Err(self.fail(
                self.pos,
                format!("the {} ended where a character was expected", self.name),
            )),
        }
    }
}

/// Input bytes for a message. Bytes outside ASCII are shown as `?`; the
/// program itself always sees them unchanged.
fn lossy(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(40)
        .map(|b| {
            if b.is_ascii_graphic() {
                *b as char
            } else {
                '?'
            }
        })
        .collect()
}
