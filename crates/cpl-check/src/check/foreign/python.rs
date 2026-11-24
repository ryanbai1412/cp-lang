//! Python habits the checker recovers: `"ab" * 3`, `a.index(x)`, `s.isdigit()`,
//! `float("inf")` and assigning a name never declared.

use super::*;

impl Checker<'_> {
    /// `"ab" * 3` is Python's repetition, which is `"ab".repeat(3)` here.
    pub(crate) fn repeat_fix(&self, s: &Expr, n: &Expr) -> Option<(Span, String)> {
        let (st, nt) = (self.text(s.span), self.text(n.span));
        let st = if st.contains(' ') && !wrapped(st) {
            format!("({st})")
        } else {
            st.to_string()
        };
        let whole = Span::new(s.span.start.min(n.span.start), s.span.end.max(n.span.end));
        Some((whole, format!("{st}.repeat({nt})")))
    }
}

/// Python's `a.index(x)` on a list: a value's first position is
/// `a.find(x)`, `None` when it isn't there, so `a.find(x)!`.
pub(crate) fn index_of(src: &str, base: Span, name: &Name, ty: &Ty) -> Option<(String, Fix)> {
    let list = ty.is_named("List");
    if name.text != "index" || !list {
        return None;
    }
    let rest = src.get(name.span.end as usize..)?.strip_prefix('(')?;
    let Some((close, true)) = first_arg(rest) else {
        return None;
    };
    let x = rest[..close].trim();
    let b = src.get(base.start as usize..base.end as usize)?;
    if x.is_empty() {
        return None;
    }
    let new = format!("{b}.find({x})!");
    let end = name.span.end + 1 + close as u32 + 1;
    let help = habits::INDEX_OF.say(&[&new]);
    let fix = Fix {
        title: habits::WRITE.say(&[&new]),
        edits: vec![(Span::new(base.start, end), new)],
    };
    Some((help, fix))
}

/// Python's `s.isdigit()`: a test of every character is `s.all(c => c.is_digit)`.
pub(crate) fn str_test(src: &str, base: Span, name: &Name, ty: &Ty) -> Option<(String, Fix)> {
    let test = match name.text.as_str() {
        "isdigit" => "is_digit",
        "isalpha" => "is_alpha",
        "isalnum" => "is_alnum",
        "isspace" => "is_space",
        _ => return None,
    };
    if *ty != Ty::Str {
        return None;
    }
    let call = if src.get(name.span.end as usize..)?.starts_with("()") {
        2
    } else {
        0
    };
    let b = src.get(base.start as usize..base.end as usize)?;
    let new = format!("{b}.all(c => c.{test})");
    let help = habits::STR_TEST.say(&[&test, &new]);
    let fix = Fix {
        title: habits::WRITE.say(&[&new]),
        edits: vec![(Span::new(base.start, name.span.end + call), new)],
    };
    Some((help, fix))
}

/// `"inf"`, `'-inf'`, `"Infinity"`: Python's spellings of an infinity, as
/// CP Lang's `inf` or `-inf`.
pub(crate) fn infinity(text: &str) -> Option<&'static str> {
    let word = text.trim_matches(['"', '\'']).to_ascii_lowercase();
    match word.trim_start_matches('+') {
        "inf" | "infinity" => Some("inf"),
        "-inf" | "-infinity" => Some("-inf"),
        _ => None,
    }
}

/// Python's `best = 0` with no `let`: every later `best = ..` and read of
/// `best` is the same root cause, so only the first assignment keeps its
/// error, and its fix declares `var` when `best` is assigned again
/// (errors/R1). Runs on the whole file, as each top-level statement is
/// checked as its own item.
/// A name another error's fix declares (`n, m = ...` as `let (n, m) =
/// ...`) is unknown below only as that error's echo.
fn drop_fix_declared(diags: &mut Vec<Diagnostic>) {
    let declared: Vec<String> = (diags.iter())
        .filter(|d| d.code != codes::UNRESOLVED_NAME)
        .flat_map(|d| d.fixes.iter().flat_map(|f| f.edits.iter()))
        .filter_map(|(_, text)| {
            ["let ", "var ", "const "]
                .iter()
                .find_map(|k| text.strip_prefix(k))
        })
        .flat_map(|rest| {
            let names = rest.split([':', '=']).next().unwrap_or_default().trim();
            let names = names.trim_start_matches('(').trim_end_matches(')');
            names
                .split(',')
                .map(|n| format!("no `{}` in scope", n.trim()))
        })
        .collect();
    let echo = |d: &Diagnostic| {
        (declared.iter()).any(|m| d.message == *m || d.message.starts_with(&format!("{m},")))
    };
    diags.retain(|d| d.code != codes::UNRESOLVED_NAME || !echo(d));
}

pub(crate) fn declare_on_first_assignment(
    mut diags: Vec<Diagnostic>,
    src: &str,
) -> Vec<Diagnostic> {
    const ASSIGNED: &str = "` is assigned before it is declared";
    let assigned = |d: &Diagnostic| {
        let name = d.message.strip_prefix('`')?.strip_suffix(ASSIGNED)?;
        (d.code == codes::UNRESOLVED_NAME).then(|| name.to_string())
    };
    let named = |d: &Diagnostic| {
        let (name, rest) = d.message.strip_prefix("no `")?.split_once('`')?;
        (d.code == codes::UNRESOLVED_NAME && rest.starts_with(" in scope"))
            .then(|| name.to_string())
    };
    drop_fix_declared(&mut diags);
    let mut first: crate::names::NameMap<Span> = Default::default();
    for d in &diags {
        if let Some(name) = assigned(d) {
            let at = first.entry(name).or_insert(d.span);
            if d.span.start < at.start {
                *at = d.span;
            }
        }
    }
    diags.retain(
        |d| match assigned(d).or_else(|| named(d)).and_then(|n| first.get(&n)) {
            Some(at) => d.span == *at || d.span.start < at.start,
            None => true,
        },
    );
    for d in &mut diags {
        let Some(name) = assigned(d) else { continue };
        let later = src.get(d.span.end as usize..).unwrap_or_default();
        if !later
            .lines()
            .skip(1)
            .any(|l| assigns(l.trim_start(), &name))
        {
            continue;
        }
        let let_ = format!("`let {name}");
        d.help = d
            .help
            .take()
            .map(|h| h.replace(&let_, &format!("`var {name}")));
        for fix in &mut d.fixes {
            fix.title = fix.title.replace("`let`", "`var`");
            for (_, text) in &mut fix.edits {
                if text == "let " {
                    *text = "var ".to_string();
                }
            }
        }
    }
    diags
}
