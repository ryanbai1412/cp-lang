//! What a source file documents.

use cpl_diag::{LineIndex, Span};
use cpl_syntax::ast::{Decorator, ExprKind, FnDecl, Stmt, StmtKind, StrPiece};

/// What kind of declaration an [`Item`] is.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Kind {
    Domain,
    Type,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Value,
    Fn,
    Field,
    Variant,
}

impl Kind {
    /// The heading its items are listed under.
    pub const fn plural(self) -> &'static str {
        match self {
            Kind::Domain => "Domains",
            Kind::Type => "Type aliases",
            Kind::Struct => "Structs",
            Kind::Enum => "Enums",
            Kind::Trait => "Traits",
            Kind::Impl => "Implementations",
            Kind::Const => "Constants",
            Kind::Value => "Values",
            Kind::Fn => "Functions",
            Kind::Field => "Fields",
            Kind::Variant => "Variants",
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Kind::Domain => "domain",
            Kind::Type => "type",
            Kind::Struct => "struct",
            Kind::Enum => "enum",
            Kind::Trait => "trait",
            Kind::Impl => "impl",
            Kind::Const => "const",
            Kind::Value => "let",
            Kind::Fn => "fn",
            Kind::Field => "field",
            Kind::Variant => "variant",
        }
    }
}

/// A fenced block in a doc comment.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Example {
    pub code: String,
    /// The 1-based line of the file its first line is on.
    pub line: u32,
}

/// One documented declaration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub kind: Kind,
    pub name: String,
    /// The declaration as written, without its body.
    pub signature: String,
    /// The doc comment, as Markdown. Empty when there is none.
    pub doc: String,
    /// The argument of `@cost`, such as `O(log n)`.
    pub cost: Option<String>,
    /// The 1-based line the declaration starts on.
    pub line: u32,
    pub examples: Vec<Example>,
    /// Fields, variants and methods.
    pub members: Vec<Item>,
}

impl Item {
    /// The first sentence of the doc comment.
    pub fn summary(&self) -> &str {
        summary(&self.doc)
    }

    /// The id of its heading on the page: `sum`, or `Prefix.sum` for a
    /// member.
    pub fn anchor(&self, owner: Option<&str>) -> String {
        match owner {
            Some(owner) => format!("{owner}.{}", self.name),
            None => self.name.clone(),
        }
    }
}

/// The first paragraph of `doc`, up to the end of its first sentence.
pub(crate) fn summary(doc: &str) -> &str {
    let paragraph = doc.split("\n\n").next().unwrap_or("");
    let line = paragraph.lines().next().unwrap_or("");
    if line.starts_with("```") {
        return "";
    }
    // A sentence ends at a period followed by a space or the paragraph's
    // end, so `a[l..r]` and `1.5` do not cut it short.
    let bytes = paragraph.as_bytes();
    let end = (0..bytes.len())
        .find(|&i| bytes[i] == b'.' && bytes.get(i + 1).is_none_or(|c| c.is_ascii_whitespace()));
    match end {
        Some(end) => &paragraph[..=end],
        None => paragraph,
    }
}

/// One source file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module {
    /// The file's name without `.cp`.
    pub name: String,
    /// The path it was read from, as given.
    pub path: String,
    /// The `///` block at the top of the file, when a blank line separates
    /// it from the first declaration.
    pub doc: String,
    pub examples: Vec<Example>,
    pub items: Vec<Item>,
}

impl Module {
    /// Every item and member, with the name of the item that owns it.
    pub fn all_items(&self) -> impl Iterator<Item = (Option<&str>, &Item)> {
        self.items.iter().flat_map(|item| {
            std::iter::once((None, item)).chain(
                item.members
                    .iter()
                    .map(move |member| (Some(item.name.as_str()), member)),
            )
        })
    }
}

/// The `///` comments of a file, by the line each is on.
struct DocComments<'a> {
    src: &'a str,
    lines: LineIndex,
    /// `(line, text after the slashes)`, in source order.
    comments: Vec<(u32, &'a str)>,
}

impl<'a> DocComments<'a> {
    fn new(src: &'a str, comments: &[Span]) -> DocComments<'a> {
        let lines = LineIndex::new(src);
        let comments = comments
            .iter()
            .filter_map(|span| {
                let text = span.text(src);
                let body = text.strip_prefix("///")?;
                // `////` is a ruler, not documentation.
                if body.starts_with('/') {
                    return None;
                }
                // A doc comment has its line to itself.
                let line_start = src[..span.start as usize]
                    .rfind('\n')
                    .map_or(0, |at| at + 1);
                if !src[line_start..span.start as usize].trim().is_empty() {
                    return None;
                }
                let body = body.strip_prefix(' ').unwrap_or(body);
                Some((lines.line_of(span.start), body.trim_end()))
            })
            .collect();
        DocComments {
            src,
            lines,
            comments,
        }
    }

    /// The run of doc comments ending on the line above `line`, and the
    /// line it starts on.
    fn above(&self, line: u32) -> (String, u32) {
        let end = self.comments.partition_point(|(at, _)| *at < line);
        let mut start = end;
        let mut expect = line;
        while start > 0 && expect > 0 && self.comments[start - 1].0 == expect - 1 {
            start -= 1;
            expect -= 1;
        }
        let text: Vec<&str> = self.comments[start..end].iter().map(|(_, t)| *t).collect();
        (text.join("\n").trim().to_string(), expect)
    }

    fn line_of(&self, offset: u32) -> u32 {
        self.lines.line_of(offset)
    }
}

/// The fenced blocks of a doc comment that starts on 0-based line `first`.
fn examples(doc: &str, first: u32) -> Vec<Example> {
    let mut out = Vec::new();
    let mut open: Option<(u32, Vec<&str>)> = None;
    for (offset, line) in doc.lines().enumerate() {
        let fence = line.trim_start().strip_prefix("```");
        match (&mut open, fence) {
            (None, Some(language)) => {
                // ```text, ```cpp and the like are shown, not checked.
                if matches!(language.trim(), "" | "cp") {
                    open = Some((first + offset as u32 + 2, Vec::new()));
                } else {
                    open = Some((0, Vec::new()));
                }
            }
            (Some(_), Some(_)) => {
                let (line, code) = open.take().unwrap();
                if line > 0 {
                    out.push(Example {
                        code: code.join("\n") + "\n",
                        line,
                    });
                }
            }
            (Some((_, code)), None) => code.push(line),
            (None, None) => {}
        }
    }
    out
}

/// `@cost("O(log n)")` or `@cost(O(1))`: the argument as written, without
/// the quotes of a string.
fn cost(decorators: &[Decorator], src: &str, base: u32) -> Option<String> {
    let decorator = decorators.iter().find(|d| d.name.text == "cost")?;
    let [arg] = decorator.args.as_slice() else {
        return None;
    };
    Some(match &arg.value.kind {
        ExprKind::Str(pieces) => pieces
            .iter()
            .map(|piece| match piece {
                StrPiece::Text(text) => text.as_str(),
                _ => "",
            })
            .collect(),
        _ => squeeze(arg.value.span.shift(base).text(src)),
    })
}

/// `text` on one line, with runs of whitespace as one space.
fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

struct Extractor<'a> {
    docs: DocComments<'a>,
    base: u32,
}

impl Extractor<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.docs.src)
    }

    /// The statement up to (not including) `stop`, on one line.
    fn head(&self, stmt: &Stmt, stop: char) -> String {
        let text = self.text(self.code_span(stmt));
        let first = text.lines().next().unwrap_or("");
        let cut = first.rfind(stop).map_or(first, |at| &first[..at]);
        squeeze(cut)
    }

    /// The statement without its decorators.
    fn code_span(&self, stmt: &Stmt) -> Span {
        let after = stmt
            .decorators
            .iter()
            .map(|d| d.span.end)
            .filter(|end| *end <= stmt.span.end)
            .max();
        match after {
            Some(after) if after > stmt.span.start => {
                let rest = &self.text(Span::new(after, stmt.span.end));
                let skipped = rest.len() - rest.trim_start().len();
                Span::new(after + skipped as u32, stmt.span.end)
            }
            _ => stmt.span,
        }
    }

    /// The 0-based line a statement starts on, decorators included.
    fn first_line(&self, stmt: &Stmt) -> u32 {
        let start = stmt
            .decorators
            .iter()
            .map(|d| d.span.start)
            .fold(stmt.span.start, u32::min);
        self.docs.line_of(start + self.base)
    }

    fn item(&self, kind: Kind, name: &str, signature: String, at: u32, doc_above: u32) -> Item {
        let (doc, first) = self.docs.above(doc_above);
        Item {
            kind,
            name: name.to_string(),
            signature,
            examples: examples(&doc, first),
            doc,
            cost: None,
            line: at + 1,
            members: Vec::new(),
        }
    }

    fn fn_item(&self, stmt: &Stmt, decl: &FnDecl) -> Item {
        let line = self.docs.line_of(self.code_span(stmt).start + self.base);
        let mut item = self.item(
            Kind::Fn,
            &decl.name.text,
            squeeze(self.text(decl.sig_span)),
            line,
            self.first_line(stmt),
        );
        item.cost = cost(&stmt.decorators, self.docs.src, self.base);
        item
    }

    fn methods(&self, stmts: &[Stmt]) -> Vec<Item> {
        stmts.iter().filter_map(|stmt| self.stmt(stmt)).collect()
    }

    fn stmt(&self, stmt: &Stmt) -> Option<Item> {
        let line = self.docs.line_of(self.code_span(stmt).start + self.base);
        let above = self.first_line(stmt);
        let item = match &stmt.kind {
            StmtKind::Fn(decl) => self.fn_item(stmt, decl),
            StmtKind::Context { name, fields } => {
                let mut item = self.item(
                    Kind::Struct,
                    &name.text,
                    self.head(stmt, ':'),
                    line,
                    above,
                );
                for field in fields {
                    let at = self.docs.line_of(field.name.span.start + self.base);
                    let signature =
                        format!("{}: {}", field.name.text, squeeze(self.text(field.ty.span)));
                    item.members
                        .push(self.item(Kind::Field, &field.name.text, signature, at, at));
                }
                item
            }
            StmtKind::Struct(decl) => {
                let mut item = self.item(
                    Kind::Struct,
                    &decl.name.text,
                    self.head(stmt, ':'),
                    line,
                    above,
                );
                for field in &decl.fields {
                    let at = self.docs.line_of(field.name.span.start + self.base);
                    let signature =
                        format!("{}: {}", field.name.text, squeeze(self.text(field.ty.span)));
                    item.members
                        .push(self.item(Kind::Field, &field.name.text, signature, at, at));
                }
                item.members.extend(self.methods(&decl.methods));
                item
            }
            StmtKind::Enum(decl) => {
                let mut item = self.item(
                    Kind::Enum,
                    &decl.name.text,
                    self.head(stmt, ':'),
                    line,
                    above,
                );
                for variant in &decl.variants {
                    let at = self.docs.line_of(variant.name.span.start + self.base);
                    let end = variant
                        .fields
                        .last()
                        .map_or(variant.name.span.end, |(_, ty)| ty.span.end);
                    let mut signature = squeeze(self.text(Span::new(variant.name.span.start, end)));
                    if !variant.fields.is_empty() {
                        signature.push(')');
                    }
                    item.members.push(self.item(
                        Kind::Variant,
                        &variant.name.text,
                        signature,
                        at,
                        at,
                    ));
                }
                item.members.extend(self.methods(&decl.methods));
                item
            }
            StmtKind::Trait { name, body, .. } => {
                let mut item =
                    self.item(Kind::Trait, &name.text, self.head(stmt, ':'), line, above);
                item.members = self.methods(&body.stmts);
                item
            }
            StmtKind::Impl { target, body, .. } => {
                let signature = self.head(stmt, ':');
                let name = squeeze(self.text(target.span));
                let mut item = self.item(Kind::Impl, &name, signature, line, above);
                item.members = self.methods(&body.stmts);
                item
            }
            StmtKind::Domain { name, .. } => self.item(
                Kind::Domain,
                &name.text,
                squeeze(self.text(self.code_span(stmt))),
                line,
                above,
            ),
            StmtKind::TypeAlias { name, .. } => self.item(
                Kind::Type,
                &name.text,
                squeeze(self.text(self.code_span(stmt))),
                line,
                above,
            ),
            StmtKind::Const { name, .. } => self.item(
                Kind::Const,
                &name.text,
                squeeze(self.text(self.code_span(stmt))),
                line,
                above,
            ),
            // A top-level `let` is part of a program, not of a library,
            // unless someone took the trouble to document it.
            StmtKind::Let { pat, .. } => {
                let name = squeeze(self.text(pat.span));
                let item = self.item(Kind::Value, &name, self.head(stmt, '='), line, above);
                if item.doc.is_empty() {
                    return None;
                }
                item
            }
            _ => return None,
        };
        // A leading underscore marks a helper.
        (!item.name.starts_with('_')).then_some(item)
    }
}

/// Reads the declarations and doc comments of one file. `path` is only
/// recorded; nothing is read from disk.
///
/// Statements the parser could not read are skipped: documentation for
/// the rest of the file is still worth having.
pub fn extract(src: &str, path: &str) -> Module {
    let lexed = cpl_syntax::lex(src);
    let parsed = cpl_syntax::parse_file(src);
    let mut extractor = Extractor {
        docs: DocComments::new(src, &lexed.comments),
        base: 0,
    };
    let mut items = Vec::new();
    let mut first_item_line = None;
    for (base, parsed_item) in &parsed.items {
        extractor.base = *base;
        for stmt in &parsed_item.stmts {
            first_item_line.get_or_insert_with(|| extractor.first_line(stmt));
            items.extend(extractor.stmt(stmt));
        }
    }

    // The module's own comment: a block that starts the file and is not
    // attached to the first declaration.
    let mut doc = String::new();
    let mut module_examples = Vec::new();
    if let Some((first, _)) = extractor.docs.comments.first() {
        let leading = src
            .lines()
            .take(*first as usize)
            .all(|line| line.trim().is_empty());
        let run_end = extractor
            .docs
            .comments
            .iter()
            .zip(*first..)
            .take_while(|((line, _), expect)| line == expect)
            .count() as u32
            + first;
        if leading && first_item_line != Some(run_end) {
            (doc, _) = extractor.docs.above(run_end);
            module_examples = examples(&doc, *first);
        }
    }

    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    Module {
        name: file.strip_suffix(".cp").unwrap_or(file).to_string(),
        path: path.to_string(),
        doc,
        examples: module_examples,
        items,
    }
}
