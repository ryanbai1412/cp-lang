//! A way a file is made to fit its size bar, after the comments and half
//! the indentation and before the spaces: the instances of one generic fn
//! whose C++ differs only in the types and the integers it names become one
//! C++ `template`, and each use names its instance, `ntt<Q0, 998244353>`.
//! Every instance is the template filled in with its own types and
//! integers, token for token, so the file means what it meant.

use crate::compact::{needs_space, tokens};
use std::collections::{BTreeMap, BTreeSet};

const BUILTIN_TYPES: &[&str] = &[
    "bool", "char", "int", "ll", "ull", "double", "size_t", "int8_t", "int16_t", "int32_t",
    "int64_t", "uint8_t", "uint16_t", "uint32_t", "uint64_t",
];

struct Tok<'a> {
    line: usize,
    start: usize,
    end: usize,
    text: &'a str,
    /// Whitespace before it.
    space: bool,
}

/// A declaration or definition at the top of a namespace, its lines and tokens.
struct Item<'a> {
    first: usize,
    last: usize,
    toks: Vec<Tok<'a>>,
    namespace: usize,
    /// A `#` line inside it: never a template's body.
    opaque: bool,
}

/// A fn definition: its item, where its name is and where its body opens.
#[derive(Clone, Copy)]
struct Def {
    item: usize,
    name: usize,
    body: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// An integer or `bool` literal: a template's `auto` parameter.
    Int,
    /// A type's one-word name: a `class` parameter.
    Type,
    Other,
}

/// Instances folded into one template: their names in file order, the last
/// of them standing for all, its tokens with each use expanded, and the
/// parameter each varying token is.
struct Fold {
    members: Vec<String>,
    stream: Vec<(usize, String)>,
    slots: Vec<Option<usize>>,
    kinds: Vec<Kind>,
}

/// What [`fold`] made of a file.
pub(crate) struct Folded {
    pub cpp: String,
    /// Each folded instance, and for the one the template stands in place
    /// of, the template's text.
    pub members: BTreeMap<String, Option<String>>,
}

/// `cpp` with the instances `origins` (a fn's C++ name to the generic fn it
/// is an instance of) says are one fn folded where their text allows.
pub(crate) fn fold(cpp: &str, origins: &BTreeMap<String, String>) -> Folded {
    let unchanged = || Folded {
        cpp: cpp.to_string(),
        members: BTreeMap::new(),
    };
    let lines: Vec<&str> = cpp.split_inclusive('\n').collect();
    let Some(items) = items(&lines) else {
        return unchanged();
    };
    let mut types: BTreeSet<&str> = BUILTIN_TYPES.iter().copied().collect();
    let mut words: BTreeSet<&str> = BTreeSet::new();
    for item in &items {
        let t: Vec<&str> = item.toks.iter().map(|t| t.text).collect();
        match t.as_slice() {
            ["struct" | "class", name, ..]
            | ["enum", "class", name, ..]
            | ["using", name, "=", ..] => {
                types.insert(name);
            }
            ["enum", name, ..] => {
                types.insert(name);
            }
            _ => {}
        }
        words.extend(
            t.iter()
                .filter(|w| w.as_bytes()[0].is_ascii_alphabetic() || w.starts_with('_')),
        );
    }
    let kind = |s: &str| -> Kind {
        let b = s.as_bytes();
        let hex = s.starts_with("0x") || s.starts_with("0X");
        if s == "true"
            || s == "false"
            || b[0].is_ascii_digit() && !s.contains('.') && (hex || !s.contains(['e', 'E']))
        {
            Kind::Int
        } else if types.contains(s) {
            Kind::Type
        } else {
            Kind::Other
        }
    };

    let mut defs: BTreeMap<&str, Def> = BTreeMap::new();
    let mut protos: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    let mut twice: BTreeSet<&str> = BTreeSet::new();
    for (at, item) in items.iter().enumerate() {
        let Some((name, def)) = declared(item, at) else {
            continue;
        };
        if !origins.contains_key(name) {
            continue;
        }
        match def {
            Some(def) => {
                if defs.insert(name, def).is_some() {
                    twice.insert(name);
                }
            }
            None => protos.entry(name).or_default().push(at),
        }
    }
    for name in twice {
        defs.remove(name);
    }
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for &name in defs.keys() {
        groups.entry(origins[name].as_str()).or_default().push(name);
    }
    groups.retain(|_, members| members.len() > 1);
    for members in groups.values_mut() {
        members.sort_by_key(|m| defs[m].item);
    }

    let expand = |def: &Def, args: &BTreeMap<String, (String, Vec<String>)>| {
        let mut out: Vec<(usize, String)> = Vec::new();
        for (j, t) in items[def.item].toks.iter().enumerate() {
            if j == def.name {
                out.push((j, "@".into()));
                continue;
            }
            match args.get(t.text) {
                Some((name, a)) => {
                    out.push((j, name.clone()));
                    if !a.is_empty() {
                        out.push((j, "<".into()));
                        for (i, x) in a.iter().enumerate() {
                            if i > 0 {
                                out.push((j, ",".into()));
                            }
                            out.push((j, x.clone()));
                        }
                        out.push((j, ">".into()));
                    }
                }
                None => out.push((j, t.text.to_string())),
            }
        }
        out
    };

    let mut args: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    let mut folds: Vec<Fold> = Vec::new();
    let mut settled = false;
    for _ in 0..16 {
        let mut next = BTreeMap::new();
        folds.clear();
        for members in groups.values() {
            let streams: Vec<Vec<(usize, String)>> =
                members.iter().map(|m| expand(&defs[m], &args)).collect();
            let mut shapes: BTreeMap<Vec<&str>, Vec<usize>> = BTreeMap::new();
            for (k, s) in streams.iter().enumerate() {
                let shape = s
                    .iter()
                    .map(|(_, x)| match kind(x) {
                        Kind::Int => "#int",
                        Kind::Type => "#type",
                        Kind::Other => x.as_str(),
                    })
                    .collect();
                shapes.entry(shape).or_default().push(k);
            }
            for class in shapes.into_values().filter(|c| c.len() > 1) {
                let rep = &streams[*class.last().unwrap()];
                let mut cols: Vec<Vec<&str>> = Vec::new();
                let mut kinds = Vec::new();
                let mut slots = vec![None; rep.len()];
                let mut ok = true;
                for p in 0..rep.len() {
                    let vals: Vec<&str> = class.iter().map(|&k| streams[k][p].1.as_str()).collect();
                    if vals.iter().all(|v| *v == vals[0]) {
                        continue;
                    }
                    let k = kind(vals[0]);
                    let prev = p.checked_sub(1).map_or("", |q| rep[q].1.as_str());
                    let after = rep.get(p + 1).map_or("", |x| x.1.as_str());
                    let qualified = matches!(prev, ":" | "." | "->" | "struct" | "enum" | "class")
                        || after == ":";
                    if k == Kind::Type && qualified || k == Kind::Int && matches!(prev, "." | "->")
                    {
                        ok = false;
                        break;
                    }
                    let c = cols.iter().position(|c| *c == vals).unwrap_or_else(|| {
                        cols.push(vals);
                        kinds.push(k);
                        cols.len() - 1
                    });
                    slots[p] = Some(c);
                }
                if !ok {
                    continue;
                }
                let names: Vec<String> = class.iter().map(|&k| members[k].to_string()).collect();
                for (i, name) in names.iter().enumerate() {
                    let a = cols.iter().map(|c| c[i].to_string()).collect();
                    next.insert(name.clone(), (names[0].clone(), a));
                }
                folds.push(Fold {
                    members: names,
                    stream: rep.clone(),
                    slots,
                    kinds,
                });
            }
        }
        if next == args {
            settled = true;
            break;
        }
        args = next;
    }
    if !settled || folds.is_empty() {
        return unchanged();
    }

    let fresh = |stem: &str| {
        let mut stem = stem.to_string();
        while words.iter().any(|w| {
            w.strip_prefix(stem.as_str())
                .is_some_and(|rest| rest.bytes().all(|c| c.is_ascii_digit()))
        }) {
            stem.push('_');
        }
        stem
    };
    let (ty, int) = (fresh("T"), fresh("V"));
    let param = |kinds: &[Kind], c: usize| match kinds[c] {
        Kind::Type => format!("{ty}{c}"),
        _ => format!("{int}{c}"),
    };

    let mut inserts: BTreeMap<usize, String> = BTreeMap::new();
    let mut gone: BTreeSet<usize> = BTreeSet::new();
    let mut edits: BTreeMap<usize, Vec<(usize, usize, String)>> = BTreeMap::new();
    let mut rendered: BTreeSet<usize> = BTreeSet::new();
    let mut members: BTreeMap<String, Option<String>> = BTreeMap::new();
    for f in &folds {
        let tmpl = &f.members[0];
        let rep = &defs[f.members.last().unwrap().as_str()];
        let header = match f.kinds.len() {
            0 => String::new(),
            n => {
                let ps: Vec<String> = (0..n)
                    .map(|c| match f.kinds[c] {
                        Kind::Type => format!("class {}", param(&f.kinds, c)),
                        _ => format!("auto {}", param(&f.kinds, c)),
                    })
                    .collect();
                format!("template<{}>", ps.join(","))
            }
        };
        let piece = |p: usize| match (f.slots[p], f.stream[p].1.as_str()) {
            (Some(c), _) => param(&f.kinds, c),
            (None, "@") => tmpl.clone(),
            (None, x) => x.to_string(),
        };
        let mut sig = header.clone();
        for p in (0..f.stream.len()).take_while(|&p| f.stream[p].0 < rep.body) {
            let x = piece(p);
            let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
            let spaced = items[rep.item].toks[f.stream[p].0].space;
            if sig.chars().last().is_some_and(|c| {
                needs_space(&c.to_string(), &x) && (spaced || word(c) && x.starts_with(word))
            }) {
                sig.push(' ');
            }
            sig.push_str(&x);
        }
        let namespace = items[rep.item].namespace;
        let earliest = items
            .iter()
            .position(|item| {
                item.namespace == namespace
                    && item
                        .toks
                        .iter()
                        .any(|t| f.members.iter().any(|m| m == t.text))
            })
            .unwrap_or(rep.item);
        inserts
            .entry(items[earliest].first)
            .or_default()
            .push_str(&format!("{sig};\n"));
        let item = &items[rep.item];
        let mut def = String::new();
        if !header.is_empty() {
            inserts
                .entry(item.first)
                .or_default()
                .push_str(&format!("{header}\n"));
            def.push_str(&format!("{header}\n"));
        }
        let mut by_tok: BTreeMap<usize, String> = BTreeMap::new();
        for (p, (tok, _)) in f.stream.iter().enumerate() {
            by_tok.entry(*tok).or_default().push_str(&piece(p));
        }
        for (j, text) in by_tok {
            let t = &item.toks[j];
            if text != t.text {
                edits
                    .entry(t.line)
                    .or_default()
                    .push((t.start, t.end, text));
            }
        }
        rendered.insert(rep.item);
        for m in &f.members {
            let d = &defs[m.as_str()];
            if d.item != rep.item {
                gone.extend(items[d.item].first..=items[d.item].last);
            }
            for &p in protos.get(m.as_str()).into_iter().flatten() {
                gone.extend(items[p].first..=items[p].last);
            }
            members.insert(m.clone(), None);
        }
        let _ = &mut def;
        members.insert(f.members.last().unwrap().clone(), Some(def));
    }
    for (at, item) in items.iter().enumerate() {
        if rendered.contains(&at) {
            continue;
        }
        for t in &item.toks {
            if let Some((name, a)) = args.get(t.text) {
                let text = match a.is_empty() {
                    true => name.clone(),
                    false => format!("{name}<{}>", a.join(",")),
                };
                edits
                    .entry(t.line)
                    .or_default()
                    .push((t.start, t.end, text));
            }
        }
    }

    let mut out = String::with_capacity(cpp.len());
    for (li, line) in lines.iter().enumerate() {
        if let Some(text) = inserts.get(&li) {
            out.push_str(text);
        }
        if gone.contains(&li) {
            continue;
        }
        let Some(es) = edits.get_mut(&li) else {
            out.push_str(line);
            continue;
        };
        es.sort();
        let mut at = 0;
        for (start, end, text) in es.iter() {
            out.push_str(&line[at..*start]);
            out.push_str(text);
            if text.ends_with('>') && line[*end..].starts_with(['>', '=']) {
                out.push(' ');
            }
            at = *end;
        }
        out.push_str(&line[at..]);
    }
    for (name, text) in members.iter_mut() {
        if let Some(text) = text {
            let def = &defs[name.as_str()];
            let item = &items[def.item];
            let mut body = String::new();
            for (li, line) in lines
                .iter()
                .enumerate()
                .take(item.last + 1)
                .skip(item.first)
            {
                let mut at = 0;
                for (start, end, t) in edits.get(&li).into_iter().flatten() {
                    body.push_str(&line[at..*start]);
                    body.push_str(t);
                    at = *end;
                }
                body.push_str(&line[at..]);
            }
            text.push_str(&body);
        }
    }
    Folded { cpp: out, members }
}

/// The items of the file, or `None` when it has a line this does not read.
fn items<'a>(lines: &[&'a str]) -> Option<Vec<Item<'a>>> {
    let mut items = Vec::new();
    let mut cur: Option<Item> = None;
    let (mut depth, mut namespace) = (0i32, 0usize);
    for (li, line) in lines.iter().enumerate() {
        let code = line.trim();
        if code.starts_with('#') {
            if let Some(item) = cur.as_mut() {
                item.opaque = true;
            }
            continue;
        }
        let ts = tokens(line.trim_end_matches('\n'))?;
        if ts.is_empty() {
            continue;
        }
        if cur.is_none() {
            let words: Vec<&str> = ts.iter().map(|t| t.1).collect();
            if matches!(words.as_slice(), ["namespace", _, "{"] | ["}"]) {
                namespace += 1;
                continue;
            }
            cur = Some(Item {
                first: li,
                last: li,
                toks: Vec::new(),
                namespace,
                opaque: false,
            });
        }
        let item = cur.as_mut().unwrap();
        item.last = li;
        for &(space, t) in &ts {
            let start = t.as_ptr() as usize - line.as_ptr() as usize;
            match t {
                "{" => depth += 1,
                "}" => depth -= 1,
                _ => {}
            }
            if depth < 0 {
                return None;
            }
            item.toks.push(Tok {
                line: li,
                start,
                end: start + t.len(),
                text: t,
                space,
            });
        }
        if depth == 0 && matches!(ts.last().unwrap().1, ";" | "}") {
            items.push(cur.take().unwrap());
        }
    }
    cur.is_none().then_some(items)
}

/// The fn an item declares, and where its body opens if it defines it:
/// `None` for anything else, or a fn whose declaration a template could not
/// repeat (a default argument, a deduced return type).
fn declared<'a>(item: &Item<'a>, at: usize) -> Option<(&'a str, Option<Def>)> {
    if item.opaque {
        return None;
    }
    let t = &item.toks;
    let open = t.iter().position(|t| t.text == "(")?;
    if open == 0 || t[0].text == "template" {
        return None;
    }
    if t[..open].iter().any(|t| {
        matches!(
            t.text,
            "auto"
                | "decltype"
                | "operator"
                | "="
                | "{"
                | "struct"
                | "class"
                | "enum"
                | "using"
                | "typedef"
                | "static_assert"
        )
    }) {
        return None;
    }
    if open >= 2 && matches!(t[open - 2].text, ":" | "." | "->" | "~") {
        return None;
    }
    let name = t[open - 1].text;
    if !name.as_bytes()[0].is_ascii_alphabetic() && !name.starts_with('_') {
        return None;
    }
    let mut depth = 0;
    let mut close = None;
    for (j, x) in t.iter().enumerate().skip(open) {
        match x.text {
            "(" => depth += 1,
            ")" => {
                depth -= 1;
                if depth == 0 {
                    close = Some(j);
                    break;
                }
            }
            "=" if depth == 1 => return None,
            _ => {}
        }
    }
    let close = close?;
    let body = t
        .iter()
        .skip(close)
        .position(|x| x.text == "{" || x.text == ";")?
        + close;
    match t[body].text {
        "{" => Some((
            name,
            Some(Def {
                item: at,
                name: open - 1,
                body,
            }),
        )),
        _ => (body == t.len() - 1).then_some((name, None)),
    }
}
