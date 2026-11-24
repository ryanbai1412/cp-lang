//! C++ habits the checker recovers: streams, iterators, `partition_point` and
//! `s.find(k) != s.end()`.

use super::*;

impl Checker<'_> {
    /// C++'s `s.find(k) != s.end()` on a set, a map or a list: `k in s`.
    pub(crate) fn keyed_find(
        &mut self,
        whole: &Expr,
        first: &Expr,
        rest: &[(CmpOp, Expr)],
    ) -> Option<Ty> {
        let [(op, end)] = rest else { return None };
        let op = match op {
            CmpOp::Ne => "in",
            CmpOp::Eq => "not in",
            _ => return None,
        };
        let (base, [key]) = method_call(first, "find")? else {
            return None;
        };
        let (other, []) = method_call(end, "end")? else {
            return None;
        };
        let text = |s: Span| &self.src[s.start as usize..s.end as usize];
        if key.name.is_some() || text(base.span) != text(other.span) {
            return None;
        }
        let base_ty = self.infer_quietly(base);
        let names = ["Set", "Map", "HashMap", "List"];
        if !matches!(&base_ty, Ty::Named { name, .. } if names.contains(&name.as_str())) {
            return None;
        }
        self.infer(&key.value);
        let new = format!("{} {op} {}", text(key.value.span), text(base.span));
        if let Some(diag) = self.report(whole.span, msgs::check::UNKNOWN_FIELD_C_S_FIND_END, &[]) {
            diag.help = Some(habits::KEYED_FIND.say(&[&new]));
            diag.fix(habits::WRITE.say(&[&new]), whole.span, new);
        }
        Some(Ty::Bool)
    }
}

/// C++'s streams: `cin >> n` is `n = read()`, and `, endl` goes, since
/// `print` ends the line.
pub(crate) fn stream_fix(src: &str, name: &str, span: Span) -> Option<(Span, String)> {
    let rest = src.get(span.end as usize..)?.split('\n').next()?.trim_end();
    match name {
        "cin" => {
            let vs: Vec<&str> = rest.strip_prefix(" >> ")?.split(" >> ").collect();
            // A name, or a place in one: `a[i]`, `g[u][v]`, `p.x`.
            let ident = |v: &&str| {
                v.starts_with(|c: char| c.is_alphabetic() || c == '_')
                    && v.chars()
                        .all(|c| c.is_alphanumeric() || "_[]. +-".contains(c))
                    && v.matches('[').count() == v.matches(']').count()
            };
            let end = span.end + rest.len() as u32;
            let place = match vs.as_slice() {
                [v] => v.to_string(),
                _ => format!("({})", vs.join(", ")),
            };
            vs.iter()
                .all(ident)
                .then(|| (Span::new(span.start, end), format!("{place} = read()")))
        }
        // `cout << a << " " << b << "\n"`: values a space apart, then the
        // line's end, is `print(a, b)`.
        "cout" => {
            let mut ops: Vec<&str> = rest.strip_prefix(" << ")?.split(" << ").collect();
            // `cout << fixed << setprecision(9) << x`: `{x:.9f}`.
            let digits = match ops.as_slice() {
                [f, p, ..] if *f == cpl_rt::cpp::FIXED => p
                    .strip_prefix("setprecision(")
                    .and_then(|d| d.strip_suffix(')'))
                    .filter(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit())),
                _ => None,
            };
            if digits.is_some() {
                ops.drain(..2);
            }
            let (end, vals) = ops.split_last()?;
            let space = |o: &str| matches!(o, "\" \"" | "' '");
            let lit = |o: &str| o.starts_with(['"', '\'']);
            let ok = matches!(*end, "\"\\n\"" | "'\\n'" | "endl")
                && vals.len() % 2 == 1
                && vals.iter().enumerate().all(|(i, o)| {
                    if i % 2 == 1 {
                        space(o)
                    } else {
                        !lit(o) && !o.is_empty()
                    }
                });
            let args: Vec<&str> = vals.iter().step_by(2).copied().collect();
            let simple = |a: &&str| a.chars().all(|c| c.is_alphanumeric() || "_.".contains(c));
            let new = match digits {
                Some(d) if args.iter().all(simple) => {
                    let f: Vec<String> = args.iter().map(|a| format!("{{{a}:.{d}f}}")).collect();
                    format!("print(\"{}\")", f.join(" "))
                }
                Some(_) => return None,
                None => format!("print({})", args.join(", ")),
            };
            ok.then(|| (Span::new(span.start, span.end + rest.len() as u32), new))
        }
        // `scanf("%d %d", &a, &b)`: plain specs, then the places read into.
        "scanf" => {
            let call = rest.strip_prefix("(\"")?.strip_suffix(')')?;
            let (format, args) = call.split_once('"')?;
            let plain = ["%d", "%i", "%ld", "%lld", "%u", "%lu", "%llu"];
            let specs: Vec<&str> = format.split_whitespace().collect();
            let args: Vec<&str> = args.strip_prefix(", ")?.split(", ").collect();
            let names: Vec<&str> = args.iter().map(|a| a.trim_start_matches('&')).collect();
            let ident =
                |v: &&str| !v.is_empty() && v.chars().all(|c| c.is_alphanumeric() || c == '_');
            let ok = specs.iter().all(|s| plain.contains(s)) && specs.len() == names.len();
            let place = match names.as_slice() {
                [v] => v.to_string(),
                _ => format!("({})", names.join(", ")),
            };
            (ok && names.iter().all(ident)).then(|| {
                (
                    Span::new(span.start, span.end + rest.len() as u32),
                    format!("{place} = read()"),
                )
            })
        }
        // `printf("%d %d\n", a, b)`: plain specs a space apart, then the
        // line's end.
        "printf" => {
            let call = rest.strip_prefix("(\"")?.strip_suffix(')')?;
            let (format, args) = call.split_once('"')?;
            let specs: Vec<&str> = format.strip_suffix("\\n")?.split(' ').collect();
            let plain = ["%d", "%i", "%ld", "%lld", "%u", "%lu", "%llu", "%s", "%c"];
            let args: Vec<&str> = args.strip_prefix(", ")?.split(", ").collect();
            let ok = specs.iter().all(|s| plain.contains(s)) && specs.len() == args.len();
            ok.then(|| {
                (
                    Span::new(span.start, span.end + rest.len() as u32),
                    format!("print({})", args.join(", ")),
                )
            })
        }
        "endl" => src
            .get(..span.start as usize)?
            .ends_with(", ")
            .then(|| (Span::new(span.start - 2, span.end), String::new())),
        _ => None,
    }
}

/// `a.begin()`, C++'s iterators: a call takes the whole sequence.
pub(crate) fn foreign_method(name: &str) -> Option<&'static str> {
    match name {
        "begin" | "end" | "rbegin" | "rend" | "cbegin" | "cend" => Some(
            "CP Lang has no iterators: a call takes the whole sequence, so `sort(a.begin(), a.end())` is `a.sort()`",
        ),
        _ => crate::builtins::list_end_help(name),
    }
}

/// C++'s and Rust's `a.partition_point(p)`: the first position where `p` turns false. CP Lang's
/// `bsearch` takes the test that turns true there, so `p` is negated.
pub(crate) fn partition_point(src: &str, name: &Name) -> Option<(String, Fix)> {
    if name.text != "partition_point" {
        return None;
    }
    let open = name.span.end as usize;
    let rest = src.get(open..)?.strip_prefix('(')?;
    let Some((close, true)) = first_arg(rest) else {
        return None;
    };
    let arg = rest[..close].trim();
    let ok = match arg.split_once("=>") {
        Some((x, body)) if !x.trim().is_empty() && !body.trim().is_empty() => {
            format!("{} => not ({})", x.trim(), body.trim())
        }
        _ if !arg.is_empty() => format!("x => not {arg}(x)"),
        _ => return None,
    };
    let end = name.span.end + 1 + close as u32 + 1;
    let new = format!("bsearch({ok})");
    let help = habits::PARTITION_POINT.say(&[&new]);
    let fix = Fix {
        title: habits::WRITE.say(&[&new]),
        edits: vec![(Span::new(name.span.start, end), new)],
    };
    Some((help, fix))
}

/// `sort(a.begin(), a.end())` as `a.sort()`, `max_element(..)` as
/// `a.max()!`, `lower_bound(.., x) - a.begin()` as `(a.first_ge(x) ?? a.len)`.
pub(crate) fn iterator_pair_fix(src: &str, base: Span, name: Span) -> Option<Fix> {
    let x = src.get(base.start as usize..base.end as usize)?;
    let rest = src
        .get(name.end as usize..)?
        .strip_prefix(&format!("(), {x}.end()"))?;
    let head = src.get(..base.start as usize)?;
    let f = [
        "sort",
        "reverse",
        "max_element",
        "min_element",
        "accumulate",
        "count",
        "lower_bound",
        "upper_bound",
        "unique",
    ]
    .into_iter()
    .find(|f| {
        head.strip_suffix(&format!("{f}("))
            .is_some_and(|h| !h.ends_with(|c: char| c.is_alphanumeric() || c == '_' || c == '.'))
    })?;
    let (arg, rest) = match rest.strip_prefix(", ") {
        Some(r) => {
            let Some((close, true)) = first_arg(r) else {
                return None;
            };
            (Some(r[..close].trim()), &r[close + 1..])
        }
        None => (None, rest.strip_prefix(')')?),
    };
    let since = format!(" - {x}.begin()");
    // `a.erase(unique(a.begin(), a.end()), a.end())`: drop repeats in place.
    let erase = format!("{x}.erase(");
    let tail = format!(", {x}.end())");
    let mut lead = 0;
    let (new, extra) = match (f, arg) {
        ("unique", None)
            if head
                .strip_suffix("unique(")
                .is_some_and(|h| h.ends_with(&erase))
                && rest.starts_with(&tail) =>
        {
            lead = erase.len() as u32;
            (format!("{x}.dedup()"), tail.len())
        }
        ("sort" | "reverse", None) => (format!("{x}.{f}()"), 0),
        ("sort", Some(g))
            if g == "greater()" || g.starts_with("greater<") && g.ends_with(">()") =>
        {
            (format!("{x}.sort(by: x => desc(x))"), 0)
        }
        ("max_element", None) => (format!("{x}.max()!"), 0),
        ("min_element", None) => (format!("{x}.min()!"), 0),
        ("accumulate", Some("0" | "0LL" | "0ll")) => (format!("{x}.sum()"), 0),
        ("count", Some(v)) => (format!("{x}.count({v})"), 0),
        // The end position when no element qualifies, as C++'s `end()`.
        ("lower_bound" | "upper_bound", Some(v)) if rest.starts_with(&since) => {
            let m = if f == "lower_bound" {
                "first_ge"
            } else {
                "first_gt"
            };
            (format!("({x}.{m}({v}) ?? {x}.len)"), since.len())
        }
        _ => return None,
    };
    let start = base.start - f.len() as u32 - 1 - lead;
    let end = (src.len() - rest.len() + extra) as u32;
    Some(Fix {
        title: habits::WRITE.say(&[&new]),
        edits: vec![(Span::new(start, end), new)],
    })
}

/// `a.begin() + i` or `a.end()` as a position in a list: the index `i`, or `a.len`.
pub(crate) fn iterator_position_fix(src: &str, base: Span, name: Span) -> Option<Fix> {
    let x = src.get(base.start as usize..base.end as usize)?;
    let rest = src.get(name.end as usize..)?.strip_prefix("()")?;
    let head = src.get(..base.start as usize)?;
    let (end, new) = match src.get(name.start as usize..name.end as usize)? {
        "begin" if rest.starts_with(&format!(", {x}.end()")) => return None,
        "begin" if rest.starts_with(" + ") => (name.end + 5, String::new()),
        "begin" => (name.end + 2, "0".to_string()),
        "end" if head.ends_with(&format!("{x}.begin(), ")) => return None,
        "end" => (name.end + 2, format!("{x}.len")),
        _ => return None,
    };
    let title = match new.is_empty() {
        true => habits::ITERATOR_INDEX.say(&[]),
        false => habits::WRITE.say(&[&new]),
    };
    Some(Fix {
        title,
        edits: vec![(Span::new(base.start, end), new)],
    })
}
