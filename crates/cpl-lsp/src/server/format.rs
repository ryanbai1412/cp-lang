//! Formatting a selection or as you type: the whole file is formatted, and
//! only the changed lines that touch the selection are sent back.

use super::{path_of_uri, position_of, range_json, uri_of, Reply, Server};
use cpl_diag::Span;
use cpl_json::{obj, Value};

impl Server {
    /// The answer to a partial formatting request, or `None` for another
    /// method.
    pub(super) fn format_part(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Option<Result<Reply, (i32, String)>> {
        let range = match method {
            "textDocument/rangeFormatting" => true,
            "textDocument/onTypeFormatting" => false,
            _ => return None,
        };
        Some(self.format_lines(params, range))
    }

    fn format_lines(&mut self, params: &Value, range: bool) -> Result<Reply, (i32, String)> {
        let uri = uri_of(params)?;
        let Some(analysis) = self.db.analyze(uri) else {
            return Ok(Reply::result(Value::Array(Vec::new())));
        };
        let offset = |at: &Value| -> Result<u32, (i32, String)> {
            Ok(analysis
                .lines
                .offset_utf16(position_of(at)?, &analysis.text))
        };
        let wanted = if range {
            Span::new(
                offset(params.at(&["range", "start"]))?,
                offset(params.at(&["range", "end"]))?,
            )
        } else {
            // The line just finished, and the one the cursor is now on.
            let at = offset(params.at(&["position"]))? as usize;
            let line = analysis.text[..at].trim_end_matches(|c| c != '\n');
            let before = line.trim_end_matches('\n').rfind('\n').map_or(0, |i| i + 1);
            Span::new(before as u32, at as u32)
        };
        let style = path_of_uri(uri)
            .map(|path| cpl_fmt::Style::discover(std::path::Path::new(&path)).0)
            .unwrap_or_default();
        let Ok(formatted) = cpl_fmt::format_with(&analysis.text, &style) else {
            return Ok(Reply::result(Value::Array(Vec::new())));
        };
        let edits = hunks(&analysis.text, &formatted)
            .into_iter()
            .filter(|(span, _)| touches(*span, wanted))
            .map(|(span, text)| {
                obj([
                    ("range", range_json(span, &analysis.text, &analysis.lines)),
                    ("newText", text.into()),
                ])
            })
            .collect();
        Ok(Reply::result(Value::Array(edits)))
    }
}

/// Whether a hunk replacing `span` touches the selection `wanted`: a line
/// that only ends where the selection starts does not.
fn touches(span: Span, wanted: Span) -> bool {
    if span.start == span.end {
        return wanted.start <= span.start && span.start <= wanted.end;
    }
    span.start < wanted.end.max(wanted.start + 1) && wanted.start < span.end
}

/// The changed runs of whole lines from `old` to `new`, each as the bytes
/// of `old` it replaces and their new text.
pub(crate) fn hunks(old: &str, new: &str) -> Vec<(Span, String)> {
    let a: Vec<&str> = old.split_inclusive('\n').collect();
    let b: Vec<&str> = new.split_inclusive('\n').collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (am, bm) = (&a[prefix..a.len() - suffix], &b[prefix..b.len() - suffix]);
    let starts: Vec<usize> = std::iter::once(0)
        .chain(a.iter().scan(0, |at, line| {
            *at += line.len();
            Some(*at)
        }))
        .collect();
    let at = |i: usize| starts[prefix + i] as u32;
    // Matched line pairs of the middle, by longest common subsequence;
    // past a size where that is slow, the middle is one hunk.
    let mut pairs = Vec::new();
    if am.len() * bm.len() <= 4_000_000 {
        let mut lcs = vec![vec![0u32; bm.len() + 1]; am.len() + 1];
        for i in (0..am.len()).rev() {
            for j in (0..bm.len()).rev() {
                lcs[i][j] = if am[i] == bm[j] {
                    lcs[i + 1][j + 1] + 1
                } else {
                    lcs[i + 1][j].max(lcs[i][j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < am.len() && j < bm.len() {
            if am[i] == bm[j] {
                pairs.push((i, j));
                i += 1;
                j += 1;
            } else if lcs[i + 1][j] >= lcs[i][j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    pairs.push((am.len(), bm.len()));
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (pi, pj) in pairs {
        if pi > i || pj > j {
            out.push((Span::new(at(i), at(pi)), bm[j..pj].concat()));
        }
        (i, j) = (pi + 1, pj + 1);
    }
    out
}
