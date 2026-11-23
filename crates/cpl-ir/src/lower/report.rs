//! Where a lowering error is shown, and what it says when it is inside
//! library code.

use super::*;

impl<'a> Lowerer<'a> {
    /// Where an error at `span` is shown. One inside library code is
    /// shown at the user's expression that asked for that code, since the
    /// file on screen is the user's.
    pub(super) fn span(&self, span: Span) -> Span {
        if self.cur_mod == 0 {
            span.shift(self.offset)
        } else {
            self.asked_at.at
        }
    }

    pub(super) fn error(&mut self, code: cpl_diag::Code, span: Span, message: impl Into<String>) {
        let at = span.shift(self.offset);
        let span = self.span(span);
        let mut message = message.into();
        let mut notes = Vec::new();
        if self.cur_mod != 0 {
            if let Some((module, line, inside)) = self.library_place(self.cur_mod, at) {
                message = format!(
                    "`{inside}` (`{module}`) does not work on the types it was given here: {message}"
                );
                notes.push(format!(
                    "{}.cp:{line}: inside `{inside}`; the gap is in `{module}`, not in your code",
                    module.replace('.', "/")
                ));
            }
            for &(module, call) in self.asked_at.via.iter().rev() {
                if let Some((module, line, inside)) = self.library_place(module, call) {
                    let file = module.replace('.', "/");
                    notes.push(format!("{file}.cp:{line}: called from `{inside}`"));
                }
            }
            self.in_library.insert(message.clone());
        }
        let diag = Diagnostic::error(code, span, message);
        self.diags
            .push(notes.into_iter().fold(diag, Diagnostic::with_note));
    }

    /// `to(x)` for an `x: from` no conversion takes. For `Int(a)` with
    /// `a: Mod<P>`, a struct whose one field already has the target type,
    /// the help names the field, `a.v`.
    pub(super) fn bad_conversion(&mut self, to: &str, from: &Ty, span: Span) {
        let message = format!("a `{}` can't be turned into a `{to}`", self.show(from));
        self.error(codes::TYPE_MISMATCH, span, message);
        let Ty::Struct(s) = from else { return };
        let [(field, ty)] = &self.structs[*s].def.fields[..] else {
            return;
        };
        let fits = matches!(
            (to, ty),
            ("Int", Ty::Int)
                | ("I128", Ty::I128)
                | ("U64", Ty::U64)
                | ("F64", Ty::Float)
                | ("Char", Ty::Char)
                | ("Bool", Ty::Bool)
        );
        if !fits {
            return;
        }
        let call = self.source(span);
        let arg = call
            .strip_prefix(to)
            .and_then(|c| c.strip_prefix('('))
            .and_then(|c| c.strip_suffix(')'))
            .map(str::trim)
            .filter(|a| a.chars().all(|c| c.is_alphanumeric() || "_.".contains(c)));
        let take = match arg {
            Some(a) => format!("`{a}.{field}`"),
            None => format!("`.{field}` on it"),
        };
        self.help(format!(
            "write {take}: a `{}` keeps its value in the field `{field}`, so naming the field is the conversion",
            self.show(from)
        ));
    }

    /// `if let`/`!` on a value that is not optional. For `if let r = i - 1`
    /// with `i: D` the offset is an `Int`, and the fix is `D.at(i - 1)`,
    /// the `?D` that is `None` off the end.
    pub(super) fn not_optional(&mut self, what: &str, value: &Expr, span: Span) {
        let message = format!(
            "`{what}` takes the value out of an optional, but this is a `{}`",
            self.show(&value.ty)
        );
        self.error(codes::TYPE_MISMATCH, span, message);
        let ExprKind::Binary(_, lhs, _) = &value.kind else {
            return;
        };
        let ExprKind::Cast(Cast::Label(d), _) = lhs.kind else {
            return;
        };
        if self.cur_mod != 0 {
            return;
        }
        let domain = self.show(&Ty::Pos(d));
        let at = format!("{domain}.at({})", self.source(span));
        self.help(format!(
            "write `{at}`: a position plus an `Int` is an `Int`, and `{domain}.at` gives back a `?{domain}` that is `None` outside `{domain}`"
        ));
        let edit = (self.span(span), at.clone());
        if let Some(last) = self.diags.last_mut() {
            last.fixes.push(cpl_diag::Fix {
                title: format!("write `{at}`"),
                edits: vec![edit],
            });
        }
    }

    /// Range ends no range takes. `Len.first..=Len.last` has `?Len` ends:
    /// the fix is `Len` itself, or `!` on each end. Two ends of one type
    /// are not a mismatch, so they never read "expected `T`, found `T`".
    pub(super) fn bad_range_ends(
        &mut self,
        (lo, hi): (Option<&ast::Expr>, Option<&ast::Expr>),
        (l, r): (&Ty, &Ty),
        inclusive: bool,
        span: Span,
    ) {
        if l != r {
            return self.mismatch(span, l, r);
        }
        let shown = self.show(l);
        let Ty::Opt(inner) = l else {
            let message =
                format!("a range's ends are `Int`s or positions of one domain, not `{shown}`s");
            return self.error(codes::TYPE_MISMATCH, span, message);
        };
        let message = format!("a range's ends can't be optional, and these are `{shown}`s");
        self.error(codes::TYPE_MISMATCH, span, message);
        let (Some(lo), Some(hi)) = (lo, hi) else {
            return;
        };
        let (lo_text, hi_text) = (self.source(lo.span), self.source(hi.span));
        let whole = match &**inner {
            Ty::Pos(d) if inclusive => {
                let d = self.show(&Ty::Pos(*d));
                (lo_text == format!("{d}.first") && hi_text == format!("{d}.last")).then_some(d)
            }
            _ => None,
        };
        let (title, help, edits) = match whole {
            Some(d) => (
                format!("write `{d}`"),
                format!("all of `{d}` is written `{d}`; `{d}.first` and `{d}.last` are `None` when `{d}` is empty"),
                vec![(self.span(span), d)],
            ),
            None => (
                "write `!` on each end".to_string(),
                format!(
                    "`{lo_text}!{}{hi_text}!` takes the values out, trapping if either is `None`",
                    if inclusive { "..=" } else { ".." }
                ),
                vec![
                    (self.span(Span::new(lo.span.end, lo.span.end)), "!".to_string()),
                    (self.span(Span::new(hi.span.end, hi.span.end)), "!".to_string()),
                ],
            ),
        };
        self.help(help);
        if self.cur_mod == 0 {
            if let Some(last) = self.diags.last_mut() {
                last.fixes.push(cpl_diag::Fix { title, edits });
            }
        }
    }

    /// `type V = 0..n` then `V.first`: a range type has no positions, a
    /// `domain` does. `range` is the alias's right side, absolute in the
    /// user file; the fix rewrites its `type` keyword.
    pub(super) fn range_alias_as_domain(&mut self, name: &str, range: Span) {
        let text = &self.mods[0].text;
        let shown = text
            .get(range.start as usize..range.end as usize)
            .unwrap_or("")
            .to_string();
        let head = text.get(..range.start as usize).unwrap_or("");
        let keyword = head.rfind("type").filter(|&at| {
            let between = head[at + 4..].split_whitespace().collect::<Vec<_>>();
            between == [name, "="]
        });
        let help = format!(
            "`type {name} = {shown}` names a range of integers; `domain {name} = {shown}` declares positions, which have `{name}.first`, `{name}.last` and index `[{name}]T`"
        );
        let Some(last) = self.diags.last_mut() else {
            return;
        };
        last.help = Some(help);
        if let Some(at) = keyword {
            let at = at as u32;
            last.fixes.push(cpl_diag::Fix {
                title: format!("declare `domain {name}`"),
                edits: vec![(Span::new(at, at + 4), "domain".into())],
            });
        }
    }

    /// The module, one-based line and innermost declaration holding `at`,
    /// a position in the current library module.
    fn library_place(&self, module: usize, at: Span) -> Option<(String, u32, String)> {
        let analysis = *self.mods.get(module)?;
        let module = &self
            .mods
            .iter()
            .flat_map(|m| m.modules())
            .find(|m| std::ptr::eq(&*m.analysis, analysis))?
            .name;
        let item = analysis
            .items
            .iter()
            .find(|item| item.span.touches(at.start))?;
        let inside = item
            .syntax
            .decls
            .iter()
            .filter(|d| d.span.shift(item.offset).touches(at.start))
            .min_by_key(|d| (d.kind != cpl_check::DeclKind::Fn, d.span.end - d.span.start))
            .or(item.syntax.decls.first())?;
        let line = analysis.lines.line_of(at.start) + 1;
        Some((module.clone(), line, inside.name.clone()))
    }
}

/// Where library code was asked for: the user's expression, the library
/// expression being lowered, and the library calls in between, outermost
/// first, which an error inside the library lists as notes.
#[derive(Clone, Default)]
pub(super) struct Asked {
    at: Span,
    lib: Span,
    via: Vec<(usize, Span)>,
}

impl Asked {
    /// Enters expression `span` of module `module`; `leave` takes the result.
    pub(super) fn enter(&mut self, span: Span, module: usize) -> (Span, Span) {
        let outer = (self.at, self.lib);
        if module == 0 {
            self.at = span;
        } else {
            self.lib = span;
        }
        outer
    }

    pub(super) fn leave(&mut self, (at, lib): (Span, Span)) {
        self.at = at;
        self.lib = lib;
    }

    /// What asks for a fn called from module `module` here.
    pub(super) fn callee(&self, module: usize) -> Asked {
        let mut via = Vec::new();
        if module != 0 {
            via = self.via.clone();
            via.push((module, self.lib));
        }
        Asked {
            at: self.at,
            lib: Span::default(),
            via,
        }
    }
}
