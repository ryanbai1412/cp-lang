// NOI 2022 Multi-Provincial Team Selection P1, Preprocessor — ideal CP Lang sketch
// Category: Unusual Verbatim Line-Oriented I/O (`read_line()`, significant whitespace & empty lines)
// Original C++: corpus/submissions/4533698.cpp (116 LOC raw, ~76 LOC stripped)
// Today-form status: NOT BUILDABLE in proto/cpc — `proto/cpc` has no `read_line()` / `getline`,
// no `Str.push` / `Str.slice` / `Str.is_alnum`, and `FastIn` discards all whitespace & empty lines!

// WISH B19-9: `read_line()` reads verbatim up to '\n' (preserving leading/trailing spaces and
// empty lines) and plays cleanly with header integers via `read_line().parse_int()!` or `read_ln()`.
let n: 1..=100 = read_line().parse_int()!
let lines: [0..n]Str = [read_line() for _ in 0..n]

fn is_ident(ch: Char) -> Bool =
    ('a' <= ch <= 'z') or ('A' <= ch <= 'Z') or ('0' <= ch <= '9') or ch == '_'

var macros: Map<Str, Str> = Map()

fn expand(line: Str, active: Set<Str>) -> Str:
    var out = ""
    var i = 0
    while i < line.len:
        // Copy non-identifier characters (spaces, punctuation, operators) verbatim
        let sep_start = i
        while i < line.len and not is_ident(line[i]):
            i += 1
        out += line[sep_start..i]
        if i == line.len: break

        // Extract maximal [A-Za-z0-9_]+ token
        let tok_start = i
        while i < line.len and is_ident(line[i]):
            i += 1
        let tok = line[tok_start..i]

        if tok not in active:
            if let body = macros.get(tok):
                var next_active = active
                next_active.insert(tok.to_str())
                out += expand(body, next_active)
                continue
        out += tok
    out

for line in lines:
    if line.starts_with("#define "):
        // Format: `#define <name> <replacement>` (replacement may contain spaces, empty, or punctuation)
        let rest = line[8..].trim_start(' ')
        let (name, body) = rest.split_once(' ') ?? (rest, "")
        macros[name] = body.trim_start(' ')
        print("")  // directive lines output an empty line
    elif line.starts_with("#undef "):
        let name = line[7..].trim(' ')
        macros.remove(name)
        print("")
    else:
        print(expand(line, Set()))

// WISH B19-3: stateful constructive `gen` for valid preprocessor programs (never `#undef`s an
// undefined macro, exercises recursive/mutual macro cycles, multi-space indentation, and empty lines)
gen(n, lines):
    let pool = ["A", "B", "C", "FOO", "BAR", "x_1", "y_2"]
    let n = rng.int(1..=min(size, 40))
    var defined: List<Str> = []
    var out: List<Str> = []
    for _ in 0..n:
        let kind = rng.int(0..4)
        if kind == 0:
            let name = rng.choice(pool)
            let body = rng.choice(
                [
                    f"{rng.choice(pool)} + {rng.choice(pool)}",
                    f"  {rng.choice(pool)}({rng.choice(pool)})  ",
                    "",
                ]
            )
            if name not in defined: defined.push(name)
            out.push(f"#define {name} {body}")
        elif kind == 1 and defined.len > 0:
            let idx = rng.int(0..defined.len)
            let name = defined.swap_remove(idx)
            out.push(f"#undef {name}")
        else:
            let sep = rng.choice([" ", "  ", "+", "(", " == "])
            out.push(f"  {rng.choice(pool)}{sep}{rng.choice(pool)} ")
    yield (n, out)
