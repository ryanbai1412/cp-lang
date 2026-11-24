// Source: NOI 2022 Provincial Selection, Preprocessor, https://dmoj.ca/problem/noimps22p1
// Problem: n <= 100 lines of at most 100 printable chars. "#define name content" and
//   "#undef name" print an empty line; other lines print with each maximal [A-Za-z0-9_] run
//   naming a defined macro replaced by its content, expanded again, except names already being
//   expanded (no recursion). Output lines have at most 1000 chars.
// Solution: keep macros in a Map. expand(line, active) copies non-identifier characters, cuts
//   each maximal identifier and, if it is a macro not in the active Set, appends its body
//   expanded with the name added to active; otherwise copies it. The bounds are tiny.
// Original C++: corpus/submissions/4533698.cpp (116 LOC raw, ~76 LOC stripped)

// WISH B19-9: `read_line()` reads verbatim up to '\n' (preserving leading/trailing spaces and
// empty lines) and plays cleanly with header integers via `read_line().parse_int()!` or `read_ln()`.
let n: 1..=100 = read_line().parse_int()!
let lines: List<Str> = [read_line() for _ in 0..n]

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
        out += line[sep_start..i].to_str()
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
        out += tok.to_str()
    out

for line in lines:
    if line.starts_with("#define "):
        // Format: `#define <name> <replacement>` (replacement may contain spaces, empty, or punctuation)
        let rest = line[8..].trim_start()
        let (name, body) = rest.split_once(' ') ?? (rest, "")
        macros[name] = body.trim_start()
        print("")  // directive lines output an empty line
    elif line.starts_with("#undef "):
        let name = line[7..].trim()
        macros.remove(name)
        print("")
    else:
        print(expand(line, Set()))

// Valid preprocessor programs: never `#undef` an undefined macro; recursive and mutual
// macro cycles, multi-space indentation and empty lines all occur.
@gen
fn input():
    let pool = ["A", "B", "C", "FOO", "BAR", "x_1", "y_2"]
    fn any() -> Str = pool[rand(0, pool.len - 1)]
    let n = rand(1, 40)
    var defined = Set<Str>()
    print(n)
    for _ in 0..n:
        let kind = rand(0, 3)
        if kind == 0:
            let name = any()
            let bodies = ["{any()} + {any()}", "  {any()}({any()})  ", ""]
            defined.insert(name.clone())
            print("#define {name} {bodies[rand(0, 2)]}")
        elif kind == 1 and defined.len > 0:
            let names = [x for x in defined]
            let name = names[rand(0, names.len - 1)].clone()
            defined.remove(name.clone())
            print("#undef {name}")
        else:
            let seps = [" ", "  ", "+", "(", " == "]
            print("  {any()}{seps[rand(0, 4)]}{any()} ")
