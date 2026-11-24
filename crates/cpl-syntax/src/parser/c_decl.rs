//! C++ declarations read off a line: `long long s = 0;` as `let s: Int = 0`.

/// A C++ type written in CP Lang, and its zero value if it has one.
pub(super) struct CType {
    ty: String,
    zero: Option<String>,
    elem: Option<Box<CType>>,
}

/// `int`, `long long`, `vector<pair<int, int>>`: the whole of `s` as a type.
pub(super) fn c_type(s: &str) -> Option<CType> {
    let s = s.trim();
    let scalar = |ty: &str, zero: Option<&str>| CType {
        ty: ty.into(),
        zero: zero.map(Into::into),
        elem: None,
    };
    let words: Vec<&str> = s.split_whitespace().collect();
    let ints = [
        "int", "long", "short", "unsigned", "signed", "ll", "ull", "size_t",
    ];
    if words
        .iter()
        .all(|w| ints.contains(w) || w.ends_with("int64_t") || w.ends_with("int32_t"))
    {
        return Some(scalar("Int", Some("0")));
    }
    match s {
        "double" | "float" | "long double" => return Some(scalar("F64", Some("0.0"))),
        "bool" => return Some(scalar("Bool", Some("false"))),
        "char" => return Some(scalar("Char", None)),
        "string" => return Some(scalar("Str", Some("\"\""))),
        _ => {}
    }
    let (head, args) = s.strip_suffix('>')?.split_once('<')?;
    // C++'s `priority_queue` pops the largest; `greater` makes it the smallest.
    if head.trim() == "priority_queue" {
        let parts = split_top(args);
        let elem = c_type(parts.first()?)?;
        let zero = match parts.as_slice() {
            [_] => "Heap(by: desc)",
            [_, _, g] if g.starts_with("greater") => "Heap()",
            _ => return None,
        };
        return Some(CType {
            ty: format!("Heap<{}>", elem.ty),
            zero: Some(zero.into()),
            elem: None,
        });
    }
    let args: Vec<CType> = split_top(args)
        .into_iter()
        .map(c_type)
        .collect::<Option<_>>()?;
    let list = |a: &[CType]| {
        a.iter()
            .map(|t| t.ty.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Some(match (head.trim(), args.as_slice()) {
        ("vector", [elem]) => CType {
            ty: format!("List<{}>", elem.ty),
            zero: Some("[]".into()),
            elem: args.into_iter().next().map(Box::new),
        },
        ("map" | "unordered_map", [_, _]) => CType {
            ty: format!("Map<{}>", list(&args)),
            zero: Some("Map()".into()),
            elem: None,
        },
        ("set" | "unordered_set", [_]) => CType {
            ty: format!("Set<{}>", list(&args)),
            zero: Some("Set()".into()),
            elem: None,
        },
        ("pair" | "tuple", [_, _, ..]) => {
            let zeros: Option<Vec<String>> = args.iter().map(|t| t.zero.clone()).collect();
            CType {
                ty: format!("({})", list(&args)),
                zero: zeros.map(|z| format!("({})", z.join(", "))),
                elem: None,
            }
        }
        _ => return None,
    })
}

/// `a, f(b, c), d` split at its top-level commas.
pub(super) fn split_top(s: &str) -> Vec<&str> {
    let (mut depth, mut from, mut parts) = (0i32, 0, Vec::new());
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(s[from..i].trim());
                from = i + 1;
            }
            _ => {}
        }
    }
    parts.push(s[from..].trim());
    parts
}

/// `vector<int>(n)` or `vector<int>(n, v)` as `List.filled(n, v)`; any
/// other value as written, its braces (`{1, 2}`) as the type's own.
pub(super) fn c_value(e: &str, ty: Option<&CType>) -> Option<String> {
    let e = e.trim();
    if let Some(inner) = e.strip_prefix('{').and_then(|e| e.strip_suffix('}')) {
        let ty = ty?;
        return Some(if ty.ty.starts_with("List<") {
            format!("[{inner}]")
        } else if ty.ty.starts_with('(') {
            format!("({inner})")
        } else {
            return None;
        });
    }
    let made = e.strip_suffix(')').and_then(|e| {
        let open = e.find(">(")? + 1;
        Some((c_type(&e[..open])?, &e[open + 1..]))
    });
    match made {
        Some((ty, args)) => c_construct(&ty, args),
        None => Some(e.to_string()),
    }
}

/// `(n)` or `(n, v)` after a vector type: `n` copies of `v` or of zero.
pub(super) fn c_construct(ty: &CType, args: &str) -> Option<String> {
    let elem = ty.elem.as_deref()?;
    let (n, fill) = match split_top(args).as_slice() {
        [n] => (n.to_string(), elem.zero.clone()?),
        [n, v] => (n.to_string(), c_value(v, Some(elem))?),
        _ => return None,
    };
    Some(format!("List.filled({n}, {fill})"))
}

/// `long long s = 0`, `int a, b` or `vector<int> a(n)`: the type's
/// length, and the line as CP Lang. A given value stays (`let`, or
/// `const` for C++'s `const`); a value
/// C++ leaves to be filled in later (`int x;`, `vector<int> a(n);`) gets
/// `var`.
/// `string w; cin >> w;` on one line: the declared names are read, so
/// `var w: Str = ""` becomes `let w: Str = read()`.
pub(super) fn c_declared_read(decl: &str, read: &str) -> Option<String> {
    let (names, zero) = decl.strip_prefix("var ")?.split_once(": ")?;
    let (ty, _) = zero.split_once(" = ")?;
    let read = read.trim().strip_suffix(';').unwrap_or(read.trim());
    let read: Vec<&str> = read
        .strip_prefix("cin")?
        .split(">>")
        .map(str::trim)
        .collect();
    let (first, read) = read.split_first()?;
    let names_read = match read {
        [one] => one.to_string(),
        _ => format!("({})", read.join(", ")),
    };
    (first.is_empty() && names_read == names).then(|| format!("let {names}: {ty} = read()"))
}

pub(super) fn c_declaration(line: &str) -> Option<(usize, String)> {
    let (keep, line) = match line.strip_prefix("const ") {
        Some(rest) => (true, rest),
        None => (false, line),
    };
    let skip = usize::from(keep) * "const ".len();
    let name_at = |s: &str| s.ends_with(|c: char| c.is_alphanumeric() || c == '_' || c == '>');
    let ty_end = (1..line.len())
        .rev()
        .filter(|&i| line.as_bytes()[i] == b' ' && name_at(&line[..i]))
        .find(|&i| c_type(&line[..i]).is_some())?;
    let ty = c_type(&line[..ty_end])?;
    let rest = line[ty_end..].trim();
    let ident = |s: &str| {
        s.starts_with(|c: char| c.is_alphabetic() || c == '_')
            && s.chars().all(|c| c.is_alphanumeric() || c == '_')
    };
    let name_len = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    let (name, tail) = rest.split_at(name_len);
    if !ident(name) {
        return None;
    }
    let tail = tail.trim();
    let t = &ty.ty;
    let decl = if tail.is_empty() && !keep {
        format!("var {name}: {t} = {}", ty.zero.as_ref()?)
    } else if let Some(value) = tail.strip_prefix('=') {
        format!(
            "{} {name}: {t} = {}",
            if keep { "const" } else { "let" },
            c_value(value, Some(&ty))?
        )
    } else if let Some(args) = tail.strip_prefix('(').and_then(|t| t.strip_suffix(')')) {
        format!("var {name}: {t} = {}", c_construct(&ty, args)?)
    } else if tail.starts_with(',') && !keep {
        let names = split_top(rest);
        if !names.iter().all(|n| ident(n)) {
            return None;
        }
        let zero = ty.zero.as_ref()?;
        let each = |s: &str| vec![s; names.len()].join(", ");
        format!(
            "var ({}): ({}) = ({})",
            names.join(", "),
            each(t),
            each(zero)
        )
    } else {
        return None;
    };
    Some((skip + ty_end, decl))
}
