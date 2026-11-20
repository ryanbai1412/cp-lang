//! The program's input: one buffer, read token by token.

/// Why a read failed, with where in the input.
pub(crate) struct InputError {
    pub message: String,
}

pub(crate) struct Input<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Input<'a> {
    pub fn new(data: &'a [u8]) -> Input<'a> {
        Input { data, pos: 0 }
    }

    fn skip_space(&mut self) {
        while self
            .data
            .get(self.pos)
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            self.pos += 1;
        }
    }

    /// Line and column of a byte of the input, both from 1.
    fn at(&self, pos: usize) -> String {
        let before = &self.data[..pos.min(self.data.len())];
        let line = before.iter().filter(|b| **b == b'\n').count() + 1;
        let col = before.iter().rev().take_while(|b| **b != b'\n').count() + 1;
        format!("input line {line}, column {col}")
    }

    pub fn fail(&self, start: usize, message: String) -> InputError {
        InputError {
            message: format!("{message} ({})", self.at(start)),
        }
    }

    /// The next run of non-space bytes, and where it starts.
    pub fn token(&mut self, what: &str) -> Result<(&'a [u8], usize), InputError> {
        self.skip_space();
        let start = self.pos;
        while self
            .data
            .get(self.pos)
            .is_some_and(|b| !b.is_ascii_whitespace())
        {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(self.fail(start, format!("the input ended where {what} was expected")));
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
        let bad = || {
            self.fail(
                start,
                format!("expected a number, found `{}`", lossy(token)),
            )
        };
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return Err(bad());
        }
        let mut value: i64 = 0;
        for digit in digits {
            let d = i64::from(digit - b'0');
            value = value
                .checked_mul(10)
                .and_then(|v| {
                    if negative {
                        v.checked_sub(d)
                    } else {
                        v.checked_add(d)
                    }
                })
                .ok_or_else(|| {
                    self.fail(
                        start,
                        format!("`{}` does not fit in an `Int`", lossy(token)),
                    )
                })?;
        }
        Ok((value, start))
    }

    pub fn float(&mut self) -> Result<f64, InputError> {
        let (token, start) = self.token("a number")?;
        std::str::from_utf8(token)
            .ok()
            .and_then(|text| text.parse::<f64>().ok())
            .ok_or_else(|| {
                self.fail(
                    start,
                    format!("expected a number, found `{}`", lossy(token)),
                )
            })
    }

    /// The next byte that is not white space.
    pub fn char(&mut self) -> Result<u8, InputError> {
        self.skip_space();
        match self.data.get(self.pos) {
            Some(byte) => {
                self.pos += 1;
                Ok(*byte)
            }
            None => Err(self.fail(
                self.pos,
                "the input ended where a character was expected".into(),
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
