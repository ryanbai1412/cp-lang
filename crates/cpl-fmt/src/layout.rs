//! Turns tokens and comments back into text.
//!
//! The work is done one logical line at a time. A logical line is what the
//! lexer puts between two `Newline` tokens: one statement or block header,
//! however many physical lines its brackets span. Each is written on one
//! line when it fits; when it does not, its longest bracket is opened up,
//! one element per line, and the same rule is applied to each element.

use crate::spacing::{self, Role};
use crate::{Comment, Tok, INDENT, MAX_WIDTH};
use cpl_syntax::{Lexed, TokenKind as K};

/// A bracket with no commas in it is opened up only when what it holds is
/// at least this wide.
const MIN_BREAK_WIDTH: usize = 30;

/// A token, or a bracket pair and what is inside it.
enum Node {
    Tok(usize),
    Group {
        open: usize,
        items: Vec<Node>,
        close: usize,
    },
}

/// One line of output, before trailing comments are lined up.
struct OutLine {
    indent: usize,
    code: String,
    comment: Option<String>,
    blank_before: bool,
}

pub(crate) fn render(src: &str, lexed: &Lexed) -> String {
    let lines = logical_lines(src, lexed);
    let mut out: Vec<OutLine> = Vec::new();
    // Source column of the open block at each depth, to place a comment
    // that sits between a block and what follows it.
    let mut widths: Vec<u32> = vec![0];
    let mut prev_depth = 0usize;
    for line in &lines {
        for comment in &line.leading {
            let depth = comment_depth(comment.col, line.depth, prev_depth, &widths);
            out.push(OutLine {
                indent: depth * INDENT,
                code: String::new(),
                comment: Some(comment.text.trim_end().to_string()),
                blank_before: comment.blank_before,
            });
        }
        if line.toks.is_empty() {
            continue;
        }
        widths.truncate(line.depth + 1);
        while widths.len() <= line.depth {
            widths.push(line.col);
        }
        widths[line.depth] = line.col;
        prev_depth = line.depth;

        let roles = spacing::roles(&line.toks);
        let mut emitter = Emitter {
            toks: &line.toks,
            roles: &roles,
            out: &mut out,
            cont: line.depth * INDENT,
            pending: None,
            at_line_start: true,
            prev: None,
        };
        emitter.start_line(line.depth * INDENT, line.blank_before);
        let nodes = tree(&line.toks);
        emitter.seq(&nodes, line.depth * INDENT);
        emitter.flush_pending();
        if let Some(comment) = line.trailing {
            out.last_mut().unwrap().comment = Some(comment.trim_end().to_string());
        }
    }
    align_comments(&mut out);

    let mut text = String::with_capacity(src.len() + src.len() / 8);
    for (i, line) in out.iter().enumerate() {
        if line.blank_before && i > 0 {
            text.push('\n');
        }
        let start = text.len();
        text.extend(std::iter::repeat_n(' ', line.indent));
        text.push_str(&line.code);
        if let Some(comment) = &line.comment {
            text.push_str(comment);
        }
        debug_assert!(!text[start..].ends_with(' '));
        text.push('\n');
    }
    text
}

/// The depth an own-line comment is written at. Between two lines of the
/// same block it follows them; after the last line of a block it stays in
/// the innermost block it was indented into.
fn comment_depth(col: u32, next_depth: usize, prev_depth: usize, widths: &[u32]) -> usize {
    if next_depth >= prev_depth {
        return next_depth;
    }
    (next_depth..=prev_depth.min(widths.len() - 1))
        .rev()
        .find(|&d| widths[d] <= col)
        .unwrap_or(next_depth)
        .max(next_depth)
}

/// Lines up the trailing comments of consecutive lines.
fn align_comments(out: &mut [OutLine]) {
    let has_both = |l: &OutLine| !l.code.is_empty() && l.comment.is_some();
    let mut i = 0;
    while i < out.len() {
        if !has_both(&out[i]) {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < out.len() && has_both(&out[j]) && !out[j].blank_before {
            j += 1;
        }
        let column = out[i..j]
            .iter()
            .map(|l| l.indent + l.code.len())
            .max()
            .unwrap()
            + 2;
        for line in &mut out[i..j] {
            let pad = column - line.indent - line.code.len();
            line.code.extend(std::iter::repeat_n(' ', pad));
        }
        i = j;
    }
}

struct Line<'a> {
    depth: usize,
    /// Source column of the first token.
    col: u32,
    blank_before: bool,
    /// Comments on their own lines before the first token.
    leading: Vec<Comment<'a>>,
    toks: Vec<Tok<'a>>,
    /// The comment after the last token.
    trailing: Option<&'a str>,
}

/// Splits the file into logical lines and hands each comment to the token
/// it belongs with.
fn logical_lines<'a>(src: &'a str, lexed: &Lexed) -> Vec<Line<'a>> {
    let bytes = src.as_bytes();
    let column = |at: usize| (at - src[..at].rfind('\n').map_or(0, |n| n + 1)) as u32;
    // Line breaks between two offsets.
    let breaks = |from: usize, to: usize| bytes[from..to].iter().filter(|b| **b == b'\n').count();

    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut current: Option<Line<'a>> = None;
    let mut depth = 0usize;
    let mut open_brackets = 0usize;
    let mut comments = lexed.comments.iter().peekable();
    // End of the last token or comment seen, and whether a token came last
    // on its line (so a comment after it trails it).
    let mut last_end = 0usize;
    let mut last_tok_end: Option<usize> = None;

    for tok in &lexed.tokens {
        match tok.kind {
            K::Indent => {
                depth += 1;
                continue;
            }
            K::Dedent => {
                depth = depth.saturating_sub(1);
                continue;
            }
            K::Newline => {
                lines.extend(current.take());
                continue;
            }
            _ => {}
        }
        let start = tok.span.start as usize;
        let mut leading: Vec<Comment<'a>> = Vec::new();
        while let Some(span) = comments.peek().filter(|c| (c.start as usize) < start) {
            let (from, to) = (span.start as usize, span.end as usize);
            let text = &src[from..to];
            let trails = last_tok_end.is_some_and(|end| breaks(end, from) == 0);
            if trails {
                // The token it trails is the last one pushed: in the
                // current line, or closing the line before.
                let owner = match current.as_mut() {
                    Some(line) => line.toks.last_mut(),
                    None => lines.last_mut().and_then(|l| l.toks.last_mut()),
                };
                owner.unwrap().trailing = Some(text);
            } else {
                leading.push(Comment {
                    text,
                    col: column(from),
                    blank_before: breaks(last_end, from) >= 2,
                });
            }
            last_end = to;
            last_tok_end = None;
            comments.next();
        }
        if tok.kind == K::Eof {
            if !leading.is_empty() {
                lines.push(Line {
                    depth: 0,
                    col: 0,
                    blank_before: false,
                    leading,
                    toks: Vec::new(),
                    trailing: None,
                });
            }
            break;
        }
        let blank_before = breaks(last_end, start) >= 2;
        let entry = Tok {
            kind: tok.kind,
            text: tok.span.text(src),
            space_before: start > last_end || last_tok_end.is_none(),
            col: column(start),
            leading: Vec::new(),
            trailing: None,
        };
        match current.as_mut() {
            Some(line) => {
                line.toks.push(entry);
                line.toks.last_mut().unwrap().leading = leading;
            }
            None if entry.kind == K::Comma && open_brackets > 0 && !lines.is_empty() => {
                // A trailing comma that closed a bracketed layout block (`=>\n    ...,\n`)
                // stays attached to the preceding line so re-lexing emits the same Dedent before `,`.
                if let Some(prev) = lines.last_mut() {
                    prev.toks.push(entry);
                    prev.toks.last_mut().unwrap().leading = leading;
                }
            }
            None => {
                let starts_closer = matches!(entry.kind, K::RParen | K::RBracket | K::RBrace);
                let line_depth = if open_brackets > 0 && !starts_closer {
                    depth.max(open_brackets)
                } else {
                    depth
                };
                current = Some(Line {
                    depth: line_depth,
                    col: entry.col,
                    blank_before,
                    leading,
                    toks: vec![entry],
                    trailing: None,
                });
            }
        }
        match tok.kind {
            K::LParen | K::LBracket | K::LBrace => open_brackets += 1,
            K::RParen | K::RBracket | K::RBrace => {
                open_brackets = open_brackets.saturating_sub(1);
            }
            _ => {}
        }
        last_end = tok.span.end as usize;
        last_tok_end = Some(last_end);
    }
    lines.extend(current.take());
    for line in &mut lines {
        if let Some(last) = line.toks.last_mut() {
            line.trailing = last.trailing.take();
        }
    }
    lines
}

/// Pairs up the brackets of one logical line.
fn tree(toks: &[Tok]) -> Vec<Node> {
    fn closer_of(kind: K) -> K {
        match kind {
            K::LParen => K::RParen,
            K::LBracket => K::RBracket,
            _ => K::RBrace,
        }
    }
    // Each open bracket and the nodes collected before it.
    let mut stack: Vec<(usize, Vec<Node>)> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    for (i, tok) in toks.iter().enumerate() {
        match tok.kind {
            K::LParen | K::LBracket | K::LBrace => {
                stack.push((i, std::mem::take(&mut nodes)));
            }
            K::RParen | K::RBracket | K::RBrace
                if stack
                    .last()
                    .is_some_and(|(open, _)| closer_of(toks[*open].kind) == tok.kind) =>
            {
                let (open, outer) = stack.pop().unwrap();
                let items = std::mem::replace(&mut nodes, outer);
                nodes.push(Node::Group {
                    open,
                    items,
                    close: i,
                });
            }
            // A closer with nothing to close is the parser's to report.
            _ => nodes.push(Node::Tok(i)),
        }
    }
    // The lexer closes every bracket it opens, so nothing is left open.
    while let Some((open, mut outer)) = stack.pop() {
        outer.push(Node::Tok(open));
        outer.append(&mut nodes);
        nodes = outer;
    }
    nodes
}

struct Emitter<'a, 'b> {
    toks: &'a [Tok<'b>],
    roles: &'a [Role],
    out: &'a mut Vec<OutLine>,
    /// Indentation of a line started in the middle of an element.
    cont: usize,
    /// A trailing comment waiting for the comma that may follow its token.
    pending: Option<&'b str>,
    at_line_start: bool,
    prev: Option<usize>,
}

impl<'a, 'b> Emitter<'a, 'b> {
    fn start_line(&mut self, indent: usize, blank_before: bool) {
        self.at_line_start = true;
        if let Some(last) = self.out.last_mut().filter(|_| self.prev.is_some()) {
            // A line started and never written to is started again.
            if last.code.is_empty() && last.comment.is_none() {
                last.indent = indent;
                return;
            }
        }
        self.out.push(OutLine {
            indent,
            code: String::new(),
            comment: None,
            blank_before,
        });
        self.at_line_start = true;
    }

    fn column(&self) -> usize {
        let line = self.out.last().unwrap();
        line.indent + line.code.len()
    }

    fn flush_pending(&mut self) -> bool {
        match self.pending.take() {
            Some(comment) => {
                self.out.last_mut().unwrap().comment = Some(comment.trim_end().to_string());
                true
            }
            None => false,
        }
    }

    fn comment_line(&mut self, indent: usize, comment: &Comment) {
        if self.at_line_start && self.out.last().is_some_and(|l| l.comment.is_none()) {
            // Reuse the empty line that was just started.
            let line = self.out.last_mut().unwrap();
            line.indent = indent;
            line.comment = Some(comment.text.trim_end().to_string());
        } else {
            self.out.push(OutLine {
                indent,
                code: String::new(),
                comment: Some(comment.text.trim_end().to_string()),
                blank_before: false,
            });
        }
    }

    /// Writes one token, with the comments attached to it.
    fn tok(&mut self, i: usize) {
        let tok = &self.toks[i];
        if tok.kind == K::Comma && self.pending.is_some() {
            // `a  // why` then `,`: the comma goes before the comment.
            self.out.last_mut().unwrap().code.push(',');
            self.prev = Some(i);
            self.flush_pending();
            self.start_line(self.cont, false);
            if let Some(comment) = tok.trailing {
                self.pending = Some(comment);
            }
            return;
        }
        if self.flush_pending() {
            self.start_line(self.cont, false);
        }
        if !tok.leading.is_empty() && self.prev.is_some() {
            for comment in &tok.leading {
                self.comment_line(self.cont, comment);
            }
            self.start_line(self.cont, false);
        }
        if !self.at_line_start {
            if let Some(p) = self.prev {
                if spacing::space_between(&self.toks[p], self.roles[p], tok, self.roles[i]) {
                    self.out.last_mut().unwrap().code.push(' ');
                }
            }
        }
        self.out.last_mut().unwrap().code.push_str(tok.text);
        self.at_line_start = false;
        self.prev = Some(i);
        if let Some(comment) = tok.trailing {
            self.pending = Some(comment);
        }
    }

    fn has_comment(&self, i: usize) -> bool {
        // The line's own leading and trailing comments were taken off its
        // first and last tokens' accounts by the caller.
        let tok = &self.toks[i];
        (i > 0 && !tok.leading.is_empty()) || tok.trailing.is_some()
    }

    /// A bracket with a comment inside cannot be written on one line.
    fn must_break(&self, node: &Node) -> bool {
        match node {
            Node::Tok(_) => false,
            Node::Group { open, items, close } => {
                self.toks[*open].trailing.is_some()
                    || !self.toks[*close].leading.is_empty()
                    || items.iter().any(|n| self.holds_comment(n))
            }
        }
    }

    fn holds_comment(&self, node: &Node) -> bool {
        match node {
            Node::Tok(i) => self.has_comment(*i),
            Node::Group { open, items, close } => {
                self.has_comment(*open)
                    || self.has_comment(*close)
                    || items.iter().any(|n| self.holds_comment(n))
            }
        }
    }

    /// Width of `nodes` written on one line, after the token `prev`.
    fn flat_width(&self, nodes: &[Node], prev: &mut Option<usize>) -> usize {
        let mut width = 0;
        for node in nodes {
            match node {
                Node::Tok(i) => width += self.tok_width(*i, prev),
                Node::Group { open, items, close } => {
                    width += self.tok_width(*open, prev);
                    let keep = items.len() - drops_trailing_comma(self.toks, items) as usize;
                    width += self.flat_width(&items[..keep], prev);
                    width += self.tok_width(*close, prev);
                }
            }
        }
        width
    }

    fn tok_width(&self, i: usize, prev: &mut Option<usize>) -> usize {
        let space = prev.is_some_and(|p| {
            spacing::space_between(&self.toks[p], self.roles[p], &self.toks[i], self.roles[i])
        });
        *prev = Some(i);
        self.toks[i].text.len() + space as usize
    }

    fn flat(&mut self, nodes: &[Node]) {
        for node in nodes {
            match node {
                Node::Tok(i) => self.tok(*i),
                Node::Group { open, items, close } => {
                    self.tok(*open);
                    let keep = items.len() - drops_trailing_comma(self.toks, items) as usize;
                    self.flat(&items[..keep]);
                    self.tok(*close);
                }
            }
        }
    }

    /// Writes `nodes`, breaking brackets where they must or where the line
    /// would run past the limit. `indent` is the indentation of the line
    /// the sequence started on.
    fn seq(&mut self, nodes: &[Node], indent: usize) {
        if nodes.is_empty() {
            return;
        }
        let forced = nodes.iter().position(|n| self.must_break(n));
        if forced.is_none() {
            let mut prev = if self.at_line_start { None } else { self.prev };
            if self.column() + self.flat_width(nodes, &mut prev) <= MAX_WIDTH {
                return self.flat(nodes);
            }
        }
        let pivot = forced.or_else(|| {
            // The widest bracket with something in it: opening it up frees
            // the most room.
            nodes
                .iter()
                .enumerate()
                .filter_map(|(n, node)| match node {
                    Node::Group { items, .. } => {
                        let width = self.flat_width(items, &mut None);
                        let is_list = top_level(self.toks, items).any(|k| k == K::Comma);
                        // Opening `a[i]` gains nothing.
                        (is_list || width >= MIN_BREAK_WIDTH).then_some((width, n))
                    }
                    _ => None,
                })
                .max()
                .map(|(_, n)| n)
        });
        let Some(pivot) = pivot else {
            // Nowhere to break: a newline outside brackets ends the statement.
            return self.flat(nodes);
        };
        self.seq(&nodes[..pivot], indent);
        let Node::Group { open, items, close } = &nodes[pivot] else {
            unreachable!()
        };
        self.broken(*open, items, *close, indent);
        self.seq(&nodes[pivot + 1..], indent);
    }

    /// Writes a bracket pair with one element per line.
    fn broken(&mut self, open: usize, items: &[Node], close: usize, indent: usize) {
        let outer_cont = self.cont;
        let inner = indent + INDENT;
        self.tok(open);
        self.cont = inner;
        let elements: Vec<&[Node]> = items
            .split_inclusive(|n| matches!(n, Node::Tok(i) if self.toks[*i].kind == K::Comma))
            .collect();
        let wants_comma = takes_trailing_comma(self.toks, items);
        for (n, element) in elements.iter().enumerate() {
            self.flush_pending();
            self.start_line(inner, false);
            self.seq(element, inner);
            let ends_with_comma =
                matches!(element.last(), Some(Node::Tok(i)) if self.toks[*i].kind == K::Comma);
            if n + 1 == elements.len() && wants_comma && !ends_with_comma {
                self.out.last_mut().unwrap().code.push(',');
            }
        }
        self.flush_pending();
        for comment in &self.toks[close].leading {
            self.comment_line(inner, comment);
            self.at_line_start = false;
        }
        self.start_line(indent, false);
        // The closer's comments were just written.
        let tok = &self.toks[close];
        self.out.last_mut().unwrap().code.push_str(tok.text);
        self.at_line_start = false;
        self.prev = Some(close);
        if let Some(comment) = tok.trailing {
            self.pending = Some(comment);
        }
        self.cont = outer_cont;
    }
}

fn top_level<'t>(toks: &'t [Tok], items: &'t [Node]) -> impl Iterator<Item = K> + 't {
    items.iter().filter_map(|n| match n {
        Node::Tok(i) => Some(toks[*i].kind),
        Node::Group { .. } => None,
    })
}

/// Whether a bracket's elements are a comma-separated list, which may end
/// in a comma, and not a comprehension or a block, which may not.
fn is_list(toks: &[Tok], items: &[Node]) -> bool {
    !top_level(toks, items).any(|k| matches!(k, K::For | K::Semi))
}

/// Written on one line, `[a, b,]` loses its last comma. `(a,)` keeps it:
/// there the comma is what makes it a tuple.
fn drops_trailing_comma(toks: &[Tok], items: &[Node]) -> bool {
    let commas = top_level(toks, items).filter(|k| *k == K::Comma).count();
    let ends_with_comma = matches!(items.last(), Some(Node::Tok(i)) if toks[*i].kind == K::Comma);
    ends_with_comma && commas >= 2 && is_list(toks, items)
}

/// Written one element per line, a list of two or more gains a last comma.
fn takes_trailing_comma(toks: &[Tok], items: &[Node]) -> bool {
    top_level(toks, items).any(|k| k == K::Comma) && is_list(toks, items)
}
