//! What a file declares, as an editor lists it: each name with its
//! signature as written, its `///` comment and, for a type, its members.
//!
//! The checker knows the types; this knows the words around them. It is
//! read off the same syntax trees the checker parsed, so an open file and
//! an imported module are outlined alike.

use cpl_syntax::ast::{FnDecl, Stmt, StmtKind, TypeKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Fn,
    Method,
    Getter,
    Field,
    Struct,
    Enum,
    Variant,
    Domain,
    Trait,
    Alias,
    Value,
    Const,
    Module,
    Keyword,
}

impl Kind {
    /// The LSP `CompletionItemKind`.
    pub fn completion_kind(self) -> u32 {
        match self {
            Kind::Method => 2,
            Kind::Fn => 3,
            Kind::Field => 5,
            Kind::Value => 6,
            Kind::Trait => 8,
            Kind::Getter => 10,
            Kind::Enum => 13,
            Kind::Domain | Kind::Alias => 25,
            Kind::Variant => 20,
            Kind::Const => 21,
            Kind::Struct => 22,
            Kind::Module => 9,
            Kind::Keyword => 14,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    /// The type as written; empty when it is left to inference.
    pub ty: String,
    /// The default as written; empty when there is none.
    pub default: String,
    /// After `;`: passed only by name.
    pub named_only: bool,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub public: bool,
    /// The declaration's first line, or a fn's whole signature, on one line.
    pub detail: String,
    pub doc: String,
    /// A fn's parameters, without `self`.
    pub params: Vec<Param>,
    /// Reached through a value (`x.len`) rather than the type (`Dir.L`).
    pub instance: bool,
    pub members: Vec<Entry>,
}

#[derive(Default, Debug)]
pub struct Outline {
    /// The module's own comment: the comment lines that open the file.
    pub doc: String,
    pub entries: Vec<Entry>,
    /// `impl ... for Target:` blocks: the target's name and what they add.
    pub impls: Vec<(String, Vec<Entry>)>,
    /// `use std.seq.{zip, scan}`: the path and the names it brings in.
    pub uses: Vec<(Vec<String>, Vec<String>)>,
}

impl Outline {
    pub fn of_source(text: &str) -> Outline {
        let parsed = cpl_syntax::parse_file(text);
        Outline::of_items(
            text,
            parsed
                .items
                .iter()
                .map(|(base, item)| (*base, &item.stmts[..])),
        )
    }

    /// Items given as the offset their spans are relative to, with their statements.
    pub fn of_items<'a>(text: &str, items: impl Iterator<Item = (u32, &'a [Stmt])>) -> Outline {
        let mut outline = Outline {
            doc: leading_comment(text),
            ..Outline::default()
        };
        for (base, stmts) in items {
            let reader = Reader { text, base };
            for stmt in stmts {
                match &stmt.kind {
                    StmtKind::Impl { target, body, .. } => {
                        if let TypeKind::Named { name, .. } = &target.kind {
                            let members = body.stmts.iter().filter_map(|s| reader.entry(s, true));
                            outline.impls.push((name.text.clone(), members.collect()));
                        }
                    }
                    StmtKind::Use { path, names, .. } => {
                        let path = path.iter().map(|n| n.text.clone()).collect();
                        outline
                            .uses
                            .push((path, names.iter().map(|n| n.text.clone()).collect()));
                    }
                    _ => {
                        // `fn f(self: T)` outside an `impl` is a method of `T` too.
                        if let StmtKind::Fn(f) = &stmt.kind {
                            let receiver = f.params.first().filter(|_| takes_self(f));
                            if let Some(TypeKind::Named { name, .. }) =
                                receiver.and_then(|p| p.ty.as_ref()).map(|t| &t.kind)
                            {
                                outline.impls.push((
                                    name.text.clone(),
                                    reader.entry(stmt, true).into_iter().collect(),
                                ));
                            }
                        }
                        outline.entries.extend(reader.entry(stmt, false))
                    }
                }
            }
        }
        outline
    }

    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Whether another module sees `member` of the type `name` declared
    /// here: a `pub` method, or a documented field. A type with no `pub`
    /// methods is plain data, and shows every field.
    pub fn offers(&self, name: &str, member: &Entry) -> bool {
        match member.kind {
            Kind::Field => {
                !member.doc.is_empty()
                    || self
                        .members_of(name)
                        .all(|m| m.kind == Kind::Field || !m.public)
            }
            _ => member.public,
        }
    }

    /// The members of the type `name` declared here, its own and its impls'.
    pub fn members_of<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Entry> + 'a {
        let own = self.find(name).into_iter().flat_map(|e| e.members.iter());
        let added = self
            .impls
            .iter()
            .filter(move |(target, _)| target == name)
            .flat_map(|(_, members)| members.iter());
        own.chain(added)
    }
}

struct Reader<'a> {
    text: &'a str,
    base: u32,
}

impl Reader<'_> {
    fn slice(&self, span: cpl_diag::Span) -> &str {
        let start = (self.base + span.start) as usize;
        let end = (self.base + span.end) as usize;
        self.text.get(start..end).unwrap_or("")
    }

    /// The line a name is declared on, without a trailing `:` that opens a body.
    fn header(&self, at: cpl_diag::Span) -> String {
        let at = (self.base + at.start) as usize;
        let start = self.text[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = self.text[at..]
            .find('\n')
            .map_or(self.text.len(), |i| at + i);
        let line = self.text[start..end].trim();
        let line = line.split(" //").next().unwrap_or(line).trim_end();
        line.strip_suffix(':')
            .unwrap_or(line)
            .trim_end()
            .to_string()
    }

    /// The `///` lines above a declaration, skipping its decorators.
    fn doc(&self, stmt: &Stmt) -> String {
        self.doc_at(stmt.span.start)
    }

    /// The `///` lines above the declaration starting at `start`.
    fn doc_at(&self, start: u32) -> String {
        let start = (self.base + start) as usize;
        // From the start of its line, past `var` or a decorator before it.
        let start = self.text[..start.min(self.text.len())]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let mut lines = Vec::new();
        for line in self.text[..start.min(self.text.len())].lines().rev() {
            let line = line.trim();
            if let Some(doc) = line.strip_prefix("///") {
                lines.push(doc.strip_prefix(' ').unwrap_or(doc));
            } else if !(line.starts_with('@') || line.is_empty() && lines.is_empty()) {
                break;
            }
        }
        // The text before a statement's own span ends on its line's indent.
        lines.reverse();
        lines.join("\n").trim().to_string()
    }

    fn entry(&self, stmt: &Stmt, in_type: bool) -> Option<Entry> {
        let entry = |name: &cpl_syntax::ast::Name, kind: Kind, detail: String| Entry {
            name: name.text.clone(),
            kind,
            public: stmt.public,
            detail,
            doc: self.doc(stmt),
            params: Vec::new(),
            instance: false,
            members: Vec::new(),
        };
        Some(match &stmt.kind {
            StmtKind::Fn(f) => {
                let getter = stmt.decorators.iter().any(|d| d.name.text == "get");
                let kind = match (in_type, getter) {
                    (true, true) => Kind::Getter,
                    (true, false) => Kind::Method,
                    _ => Kind::Fn,
                };
                let mut e = entry(&f.name, kind, squash(self.slice(f.sig_span)));
                e.instance = in_type && takes_self(f);
                e.params = f
                    .params
                    .iter()
                    .filter(|p| p.name.text != "self")
                    .map(|p| Param {
                        name: p.label_text().to_string(),
                        ty: p
                            .ty
                            .as_ref()
                            .map_or(String::new(), |t| squash(self.slice(t.span))),
                        default: p
                            .default
                            .as_ref()
                            .map_or(String::new(), |d| squash(self.slice(d.span))),
                        named_only: p.named_only,
                    })
                    .collect();
                e
            }
            StmtKind::Context { name, fields } => {
                let mut e = entry(name, Kind::Struct, self.header(name.span));
                e.members = fields
                    .iter()
                    .map(|f| Entry {
                        name: f.name.text.clone(),
                        kind: Kind::Field,
                        public: true,
                        detail: format!("{}: {}", f.name.text, squash(self.slice(f.ty.span))),
                        doc: self.doc_at(f.name.span.start),
                        params: Vec::new(),
                        instance: true,
                        members: Vec::new(),
                    })
                    .collect();
                e
            }
            StmtKind::Struct(s) => {
                let mut e = entry(&s.name, Kind::Struct, self.header(s.name.span));
                let line_of = |at: cpl_diag::Span| {
                    let at = ((self.base + at.start) as usize).min(self.text.len());
                    self.text[..at].rfind('\n')
                };
                e.members = s
                    .fields
                    .iter()
                    .map(|f| Entry {
                        name: f.name.text.clone(),
                        kind: Kind::Field,
                        public: true,
                        detail: format!("{}: {}", f.name.text, squash(self.slice(f.ty.span))),
                        // A parameter on the struct's own line has the struct's doc.
                        doc: if line_of(f.name.span) == line_of(s.name.span) {
                            String::new()
                        } else {
                            self.doc_at(f.name.span.start)
                        },
                        params: Vec::new(),
                        instance: true,
                        members: Vec::new(),
                    })
                    .chain(s.methods.iter().filter_map(|m| self.entry(m, true)))
                    .collect();
                e
            }
            StmtKind::Enum(en) => {
                let mut e = entry(&en.name, Kind::Enum, self.header(en.name.span));
                e.members = en
                    .variants
                    .iter()
                    .map(|v| {
                        let fields: Vec<&str> =
                            v.fields.iter().map(|(_, t)| self.slice(t.span)).collect();
                        let detail = if fields.is_empty() {
                            v.name.text.clone()
                        } else {
                            format!("{}({})", v.name.text, fields.join(", "))
                        };
                        Entry {
                            name: v.name.text.clone(),
                            kind: Kind::Variant,
                            public: true,
                            detail,
                            doc: String::new(),
                            params: Vec::new(),
                            instance: false,
                            members: Vec::new(),
                        }
                    })
                    .chain(en.methods.iter().filter_map(|m| self.entry(m, true)))
                    .collect();
                e
            }
            StmtKind::Trait { name, body, .. } => {
                let mut e = entry(name, Kind::Trait, self.header(name.span));
                e.members = body
                    .stmts
                    .iter()
                    .filter_map(|m| self.entry(m, true))
                    .collect();
                e
            }
            StmtKind::Domain { name, .. } => entry(name, Kind::Domain, self.header(name.span)),
            StmtKind::TypeAlias { name, .. } => entry(name, Kind::Alias, self.header(name.span)),
            StmtKind::Const { name, .. } => entry(name, Kind::Const, self.header(name.span)),
            StmtKind::Let { pat, .. } if !in_type => match &pat.kind {
                cpl_syntax::ast::PatternKind::Name(text) => {
                    let name = cpl_syntax::ast::Name {
                        text: text.clone(),
                        span: pat.span,
                    };
                    entry(&name, Kind::Value, self.header(pat.span))
                }
                _ => return None,
            },
            _ => return None,
        })
    }
}

fn takes_self(f: &FnDecl) -> bool {
    f.params.first().is_some_and(|p| p.name.text == "self")
}

/// Text written over several lines, on one.
fn squash(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if !out.is_empty() && !out.ends_with(['(', '[']) && !word.starts_with([')', ']', ',']) {
            out.push(' ');
        }
        out.push_str(word);
    }
    out.replace(",)", ")")
}

fn leading_comment(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .take_while(|l| l.starts_with("//"))
        .map(|l| l.trim_start_matches('/').trim_start_matches('!').trim())
        .collect();
    lines.join("\n").trim().to_string()
}
