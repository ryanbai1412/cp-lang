//! What a call or loop costs: documented `O(..)` costs evaluated at the
//! problem's sizes, for hovers and lenses.

use super::*;

/// A documented cost's value when its size `n` is `n`, logs base 2: `O(n
/// log n)`, `O(n^2)`, `O(log^2 n)`; `None` for any other name in it.
pub(super) fn steps_of(cost: &str, n: f64) -> Option<f64> {
    let body = cpl_syntax::doc::big_o(cost)?;
    let mut tokens = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_alphanumeric() {
            let mut word = String::new();
            while let Some(&c) = chars.peek().filter(|c| c.is_alphanumeric()) {
                word.push(c);
                chars.next();
            }
            tokens.push(word);
        } else {
            tokens.push(c.to_string());
            chars.next();
        }
    }
    let mut at = 0;
    let value = steps_sum(&tokens, &mut at, n)?;
    (at == tokens.len() && value.is_finite()).then_some(value.max(1.0))
}

fn steps_sum(t: &[String], at: &mut usize, n: f64) -> Option<f64> {
    let mut total = steps_product(t, at, n)?;
    while t.get(*at).is_some_and(|s| s == "+") {
        *at += 1;
        total += steps_product(t, at, n)?;
    }
    Some(total)
}

fn steps_product(t: &[String], at: &mut usize, n: f64) -> Option<f64> {
    let mut total = steps_power(t, at, n)?;
    loop {
        match t.get(*at).map(String::as_str) {
            Some("*") => *at += 1,
            Some("+") | Some(")") | None => return Some(total),
            _ => {}
        }
        total *= steps_power(t, at, n)?;
    }
}

fn steps_exponent(t: &[String], at: &mut usize) -> Option<i32> {
    if t.get(*at).is_some_and(|s| s == "^") {
        *at += 1;
        let e = t.get(*at)?.parse().ok()?;
        *at += 1;
        return Some(e);
    }
    Some(1)
}

fn steps_power(t: &[String], at: &mut usize, n: f64) -> Option<f64> {
    let token = t.get(*at)?.clone();
    *at += 1;
    let base = match token.as_str() {
        "n" => n,
        "log" => {
            let e = steps_exponent(t, at)?;
            return Some(steps_power(t, at, n)?.log2().max(1.0).powi(e));
        }
        "(" => {
            let inner = steps_sum(t, at, n)?;
            (t.get(*at)? == ")").then_some(())?;
            *at += 1;
            inner
        }
        number => number.parse().ok()?,
    };
    Some(base.powi(steps_exponent(t, at)?))
}

/// The steps of a cost written in domain sizes, `O(log |V|)`, at their most
/// positions; none if it names anything else.
fn domain_steps(cost: &str, domains: &[(String, u128)]) -> Option<f64> {
    let mut cost = cost.to_string();
    for (d, size) in domains {
        cost = cost.replace(&format!("|{d}|"), &format!("({size})"));
    }
    let names_n = cost
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|w| w == "n");
    (!cost.contains('|') && !names_n)
        .then(|| steps_of(&cost, 0.0))
        .flatten()
}

/// `cost` with each `xs.len` written as the bound on the list `xs` passed
/// at the nearest argument after `after`, in the source's terms and as a
/// number; a `.len` with no bound is left as it is.
fn lens_in(cost: &str, after: u32, text: &str, sizes: &[cpl_lint::ProvenSize]) -> (String, String) {
    let (mut shown, mut numeric) = (cost.to_string(), cost.to_string());
    let words = cost.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.');
    for list in words.filter_map(|w| w.strip_suffix(".len")) {
        let argument = |s: &&cpl_lint::ProvenSize| {
            let (start, end) = (s.receiver.0 as usize, s.receiver.1 as usize);
            s.receiver.0 >= after
                && text.get(start..end) == Some(list)
                && !text[end..].starts_with('.')
        };
        let Some(size) = sizes.iter().filter(argument).min_by_key(|s| s.receiver.0) else {
            continue;
        };
        let count = match size.count.contains(' ') {
            true => format!("({})", size.count),
            false => size.count.clone(),
        };
        shown = shown.replace(&format!("{list}.len"), &count);
        numeric = numeric.replace(&format!("{list}.len"), &format!("({})", size.max));
    }
    (shown, numeric)
}

/// A documented cost with its size `n` written as `size`: `O(n log n)` for
/// `ys.len` is `O(ys.len log ys.len)`.
fn size_in(cost: &str, size: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for c in cost.chars().chain(std::iter::once('\0')) {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        out += if word == "n" { size } else { &word };
        word.clear();
        if c != '\0' {
            out.push(c);
        }
    }
    out
}

/// The steps of one call at max input, and its `O(..)` when that is short.
pub(super) struct Steps {
    pub(super) steps: f64,
    pub(super) cost: Option<String>,
}

/// A call with a documented cost that can be evaluated: its span, the
/// steps it takes per call of the fn it is in, and its `O(..)` term.
struct Evaluated {
    span: Span,
    steps: f64,
    term: Option<String>,
}

/// The calls in `f`'s body with a documented cost, each with how often it
/// runs per call: `.sorted O(n log n) n * q = 2e8 times`; and the steps of
/// one call at max input, with its `O(..)` when that is short, if every
/// loop has a count and every call a documented cost that can be evaluated.
pub(super) fn fn_call_costs(
    db: &mut cpl_check::Session,
    uri: &str,
    f: &cpl_lint::ProvenFn,
    facts: &Proven,
) -> (Vec<String>, Option<Steps>) {
    const MOST: usize = 4;
    const LONGEST: usize = 100;
    let (mut out, evaluated, documented) = fn_documented(db, uri, f, facts);
    if out.len() > MOST {
        let more = out.len() - MOST;
        out.truncate(MOST);
        out.push(format!("and {more} more"));
    }
    let extra = fn_extra(db, uri, f, &evaluated, facts, 0);
    let total = extra.and_then(|(steps, terms)| {
        let loops = f.steps?;
        let shown = loops > 0 || documented || terms.as_ref().is_some_and(|t| !t.is_empty());
        let terms = terms.map(|t| match (loops, &f.count) {
            (0, _) => Some(t),
            (_, Some(c)) => Some(std::iter::once(c.clone()).chain(t).collect()),
            (_, None) => None,
        });
        shown.then_some((loops as f64 + steps, terms.flatten()))
    });
    let total = total.map(|(steps, terms)| {
        let cost = terms
            .map(|t: Vec<String>| format!("O({})", t.join(" + ")))
            .filter(|c| c.len() <= LONGEST);
        Steps { steps, cost }
    });
    (out, total)
}

/// The calls in `f`'s body with a documented cost, each described with how
/// often it runs; the steps of those that can be evaluated, by span, with
/// their `O(..)` terms; and whether any call has a documented cost.
fn fn_documented(
    db: &mut cpl_check::Session,
    uri: &str,
    f: &cpl_lint::ProvenFn,
    facts: &Proven,
) -> (Vec<String>, Vec<Evaluated>, bool) {
    let mut out = Vec::new();
    let mut evaluated = Vec::new();
    let costs = db.costs_in(uri, f.body.0, f.body.1);
    let text = facts.text();
    let sizes = if costs.is_empty() {
        &[][..]
    } else {
        facts.sizes()
    };
    let domains = if costs.iter().any(|(_, _, c)| c.contains('|')) {
        facts.domains()
    } else {
        &[][..]
    };
    for (span, name, cost) in &costs {
        let receiver = name
            .starts_with('.')
            .then(|| receiver_before(text, span.start as usize))
            .flatten();
        let size = sizes.iter().find(|s| s.receiver.1 + 1 == span.start);
        let (cost, numeric) = match name.starts_with('.') {
            true => (cost.clone(), cost.clone()),
            false => lens_in(cost, span.end, text, sizes),
        };
        let cost = &cost;
        let steps = match size {
            Some(s) => steps_of(cost, s.max as f64),
            None => domain_steps(&numeric, domains),
        };
        let (name, cost) = match (receiver, size) {
            (Some(r), Some(s)) if s.count.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                (format!("{r}{name}"), size_in(cost, &s.count))
            }
            (Some(r), Some(s)) => (
                format!("{r}{name}"),
                size_in(cost, &format!("({})", s.count)),
            ),
            (Some(r), None) => (format!("{r}{name}"), size_in(cost, &format!("{r}.len"))),
            (None, _) => (name.clone(), cost.clone()),
        };
        let inside = |r: &&cpl_lint::Run| r.span.0 <= span.start && span.end <= r.span.1;
        let run = f
            .runs
            .iter()
            .filter(inside)
            .min_by_key(|r| r.span.1 - r.span.0);
        let times = match run {
            None => "once".to_string(),
            Some(r) if r.times == Some(0) => continue,
            Some(cpl_lint::Run { times: Some(1), .. }) => "once".to_string(),
            Some(cpl_lint::Run {
                times: Some(t),
                count,
                ..
            }) => match count {
                Some(c) if !c.chars().all(|c| c.is_ascii_digit()) => {
                    format!("{c} = {} times", count_text(*t))
                }
                _ => format!("{} times", count_text(*t)),
            },
            Some(_) => "an unknown number of times".to_string(),
        };
        if let (Some(s), Some(r)) = (steps, run.map_or(Some(1), |r| r.times)) {
            let inner = cpl_syntax::doc::big_o(&cost);
            let term = match (run.and_then(|r| r.count.as_deref()), inner) {
                (_, None) => None,
                (None, Some(i)) => Some(i.to_string()),
                (Some(c), Some(i)) if c.contains(' ') => Some(format!("({c}) * ({i})")),
                (Some(c), Some(i)) => Some(format!("{c} * ({i})")),
            };
            evaluated.push(Evaluated {
                span: *span,
                steps: s * r as f64,
                term,
            });
        }
        let cost = match steps {
            Some(s) if times == "once" => {
                format!("{cost}, about {} steps,", count_text(s.ceil() as u128))
            }
            Some(s) => format!("{cost}, about {} steps each,", count_text(s.ceil() as u128)),
            None => cost,
        };
        out.push(format!("{name} {cost} {times}"));
    }
    (out, evaluated, !costs.is_empty())
}

/// The steps one call of `f` takes at max input beyond the loop iterations
/// `f.steps` counts: each call's steps times how often it runs, a
/// documented `O(1)` call counting one step, and the same for each fn in
/// the file it calls; `None` if a loop or call is uncounted. With it, the
/// `O(..)` terms of the calls with a stated cost.
fn fn_extra(
    db: &mut cpl_check::Session,
    uri: &str,
    f: &cpl_lint::ProvenFn,
    evaluated: &[Evaluated],
    facts: &Proven,
    depth: usize,
) -> Option<(f64, Option<Vec<String>>)> {
    f.steps?;
    if depth > 16 || f.runs.iter().any(|r| r.times.is_none()) {
        return None;
    }
    let mut steps = 0.0;
    let mut terms = Some(Vec::new());
    for &(start, end) in &f.callees {
        if let Some(Evaluated { steps: s, term, .. }) = evaluated
            .iter()
            .find(|e| e.span.start == start && e.span.end == end)
        {
            steps += s;
            terms = terms.zip(term.clone()).map(|(mut t, term)| {
                t.push(term);
                t
            });
            continue;
        }
        let hover = db.hover(uri, start)?;
        let cost = hover.cost.as_deref()?;
        if (hover.span.start, hover.span.end) != (start, end)
            || cpl_syntax::doc::big_o(cost) != Some("1")
        {
            return None;
        }
        let inside = |r: &&cpl_lint::Run| r.span.0 <= start && end <= r.span.1;
        let run = f
            .runs
            .iter()
            .filter(inside)
            .min_by_key(|r| r.span.1 - r.span.0);
        steps += run.map_or(Some(1), |r| r.times)? as f64;
    }
    for &(start, end) in &f.builds {
        let inside = |r: &&cpl_lint::Run| r.span.0 <= start && end <= r.span.1;
        let run = f
            .runs
            .iter()
            .filter(inside)
            .min_by_key(|r| r.span.1 - r.span.0);
        steps += run.map_or(Some(1), |r| r.times)? as f64;
    }
    for helper in &f.helpers {
        let callee = facts
            .fns()
            .iter()
            .find(|g| g.name == helper.callee)?
            .clone();
        let (_, evaluated, _) = fn_documented(db, uri, &callee, facts);
        let (each, each_terms) = fn_extra(db, uri, &callee, &evaluated, facts, depth + 1)?;
        steps += each * helper.times? as f64;
        let each_terms = each_terms.filter(|t| !t.is_empty()).map(|t| t.join(" + "));
        terms = match (terms, each_terms, &helper.count) {
            (terms, None, _) => terms,
            (Some(mut t), Some(each), None) => {
                t.push(each);
                Some(t)
            }
            (Some(mut t), Some(each), Some(c)) => {
                let c = if c.contains(' ') {
                    format!("({c})")
                } else {
                    c.clone()
                };
                t.push(format!("{c} * ({each})"));
                Some(t)
            }
            _ => None,
        };
    }
    Some((steps, terms))
}

/// `n`, and from 1e4 on its size too: `200000 (2e5)`, `249999500000 (2.5e11)`.
pub(crate) fn count_text(n: u128) -> String {
    if n < 10_000 {
        return n.to_string();
    }
    let digits = n.to_string();
    let exp = digits.len() - 1;
    let tenths = (n as f64 / 10f64.powi(exp as i32) * 10.0).round() as u64;
    let (tenths, exp) = if tenths >= 100 {
        (10, exp + 1)
    } else {
        (tenths, exp)
    };
    let mantissa = if tenths % 10 == 0 {
        (tenths / 10).to_string()
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    };
    format!("{n} ({mantissa}e{exp})")
}
