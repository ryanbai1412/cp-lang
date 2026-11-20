//! Byte offsets to lines and columns, in both directions.

/// A zero-based line and column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}

/// Line starts of one source text. Building it is one pass over the bytes;
/// lookups are a binary search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    /// Byte offset of the first character of each line.
    starts: Vec<u32>,
    len: u32,
}

impl LineIndex {
    pub fn new(text: &str) -> LineIndex {
        let mut starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i as u32 + 1);
            }
        }
        LineIndex {
            starts,
            len: text.len() as u32,
        }
    }

    pub fn line_count(&self) -> u32 {
        self.starts.len() as u32
    }

    /// The zero-based line holding `offset`.
    pub fn line_of(&self, offset: u32) -> u32 {
        match self.starts.binary_search(&offset) {
            Ok(line) => line as u32,
            Err(next) => next as u32 - 1,
        }
    }

    pub fn line_start(&self, line: u32) -> u32 {
        self.starts.get(line as usize).copied().unwrap_or(self.len)
    }

    /// The byte range of `line`, without its newline.
    pub fn line_range(&self, line: u32, text: &str) -> (u32, u32) {
        let start = self.line_start(line);
        let mut end = self
            .starts
            .get(line as usize + 1)
            .map_or(self.len, |next| next - 1);
        if end > start && text.as_bytes()[end as usize - 1] == b'\r' {
            end -= 1;
        }
        (start, end.max(start))
    }

    /// Line and column in bytes.
    pub fn line_col(&self, offset: u32) -> LineCol {
        let offset = offset.min(self.len);
        let line = self.line_of(offset);
        LineCol {
            line,
            col: offset - self.line_start(line),
        }
    }

    /// Line and column in UTF-16 code units, which is what LSP clients send
    /// and expect by default.
    pub fn utf16(&self, offset: u32, text: &str) -> LineCol {
        let offset = floor_char_boundary(text, offset.min(self.len) as usize) as u32;
        let line = self.line_of(offset);
        let start = self.line_start(line) as usize;
        let col = text[start..offset as usize].encode_utf16().count() as u32;
        LineCol { line, col }
    }

    /// The byte offset of a UTF-16 position. Positions past the end of a
    /// line or of the file clamp, as the LSP specification asks.
    pub fn offset_utf16(&self, pos: LineCol, text: &str) -> u32 {
        if pos.line >= self.line_count() {
            return self.len;
        }
        let (start, end) = self.line_range(pos.line, text);
        let mut units = 0;
        for (i, ch) in text[start as usize..end as usize].char_indices() {
            if units >= pos.col {
                return start + i as u32;
            }
            units += ch.len_utf16() as u32;
        }
        end
    }
}

fn floor_char_boundary(text: &str, mut at: usize) -> usize {
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_map_to_lines_and_columns() {
        let text = "let a = 1\nlet b = 2\n\nprint(a)";
        let idx = LineIndex::new(text);
        assert_eq!(idx.line_count(), 4);
        assert_eq!(idx.line_col(0), LineCol { line: 0, col: 0 });
        assert_eq!(idx.line_col(10), LineCol { line: 1, col: 0 });
        assert_eq!(idx.line_col(14), LineCol { line: 1, col: 4 });
        assert_eq!(idx.line_col(20), LineCol { line: 2, col: 0 });
        assert_eq!(idx.line_col(text.len() as u32), LineCol { line: 3, col: 8 });
        assert_eq!(idx.line_range(1, text), (10, 19));
        assert_eq!(idx.line_range(2, text), (20, 20));
    }

    #[test]
    fn utf16_columns_round_trip_through_wide_characters() {
        // 'é' is 2 bytes and 1 UTF-16 unit; '𝄞' is 4 bytes and 2 units.
        let text = "// é𝄞x\nlet a = 1";
        let idx = LineIndex::new(text);
        let x = text.find('x').unwrap() as u32;
        let pos = idx.utf16(x, text);
        assert_eq!(pos, LineCol { line: 0, col: 6 });
        assert_eq!(idx.offset_utf16(pos, text), x);
        let a = text.find("a =").unwrap() as u32;
        assert_eq!(idx.offset_utf16(idx.utf16(a, text), text), a);
    }

    #[test]
    fn positions_past_the_end_clamp() {
        let text = "ab\ncd";
        let idx = LineIndex::new(text);
        assert_eq!(idx.offset_utf16(LineCol { line: 0, col: 99 }, text), 2);
        assert_eq!(idx.offset_utf16(LineCol { line: 9, col: 0 }, text), 5);
        assert_eq!(idx.line_col(99), LineCol { line: 1, col: 2 });
    }

    #[test]
    fn crlf_lines_end_before_the_carriage_return() {
        let text = "ab\r\ncd";
        let idx = LineIndex::new(text);
        assert_eq!(idx.line_range(0, text), (0, 2));
        assert_eq!(idx.offset_utf16(LineCol { line: 0, col: 9 }, text), 2);
    }
}
