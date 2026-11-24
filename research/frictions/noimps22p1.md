# Friction Report: noimps22p1

The v0.2 port scanned each line with an index and two inner `while` loops, one for separator
characters and one for identifier characters, spelled `is_ident` as four character ranges, and
cut directives with the magic offsets `line[8..]` and `line[7..]`. Now `expand` walks
`line.runs(by: is_ident)` and treats every run alike (a separator run is never a macro name),
`is_ident` is `ch.is_alnum or ch == '_'`, and the directive prefixes are named. Adds tests.toml.

## No `strip_prefix` on `Str`
- **Where:** `line.starts_with(define)` then `line[define.len..]`.
- **Ideal:** `if let rest = line.strip_prefix("#define "):`.

## A recursive call with one more set element needs a copy
- **Where:** `var inner = active; inner.insert(word.clone())` before `expand(body, inner)`.
- **Ideal:** `expand(body, active + {word})`, or a scoped insert that undoes itself.

## The header count is read as a line
- **Where:** `read_line().parse_int()!`, which the `// Bounds:` gate does not see as a read.
- **Ideal:** `let n: 1..=100 = read()` followed by verbatim `read_line()` calls.

## Dream
```cp
let n: 1..=100 = read()
fn expand(line: Str, active: Set<Str>) -> Str =
    "".join(if let body = macros.get(w) and w not in active: expand(body, active + {w}) else: w
            for w in line.runs(by: c => c.is_alnum or c == '_'))
for line in [read_line() for _ in 0..n]:
    if let rest = line.strip_prefix("#define "): macros.insert(rest.split_once(' ') ?? (rest, "")); print("")
    elif let name = line.strip_prefix("#undef "): macros.remove(name.trim()); print("")
    else: print(expand(line, [:]))
```

- Tokens: port 577, original 757
- Max input (`gen.py --max`): port 0.00 s, 3328 KB, original 0.00 s, 3712 KB
