// Source: NOI 2022 Provincial Selection, Preprocessor, https://dmoj.ca/problem/noimps22p1
// limits: 1000 ms, 512 MB
// Problem: n <= 100 lines of at most 100 printable chars. "#define name content" and
//   "#undef name" print an empty line; other lines print with each maximal [A-Za-z0-9_] run
//   naming a defined macro replaced by its content, expanded again, except names already being
//   expanded (no recursion). Output lines have at most 1000 chars.
// Solution: keep macros in a Map. expand(line) walks the line's runs of identifier
//   and other characters; a run naming a macro not being expanded becomes its body,
//   expanded with the name marked active until it returns; every other run is copied. The bounds are tiny.
// Bounds: n: 1..=100
// Original C++: corpus/submissions/4533698.cpp (116 LOC raw, ~76 LOC stripped)
let n = read_line().parse<1..=100>()!
let lines: List<Str> = [read_line() for _ in 0..n]

fn is_ident(ch: Char) -> Bool = ch.is_alnum or ch == '_'

var macros: Map<Str, Str> = Map()

var active = Set<Str>()  // the macros being expanded right now

fn expand(line: Str) -> Str:
    var out = ""
    for run in line.runs(by: is_ident):
        let word = run.to_str()
        if let body = macros.get(word) and word not in active:
            active.insert(word)
            out += expand(body)
            active.remove(run.to_str())
        else:
            out += word
    out

let (define, undef) = ("#define ", "#undef ")
for line in lines:
    if line.starts_with(define):
        let rest = line[define.len..].trim_start()
        let (name, body) = rest.split_once(' ') ?? (rest, "")
        macros[name] = body.trim_start()
        print("")
    elif line.starts_with(undef):
        macros.remove(line[undef.len..].trim())
        print("")
    else:
        print(expand(line))

// Valid preprocessor programs: never `#undef` an undefined macro; recursive and mutual
// macro cycles, multi-space indentation and empty lines all occur.
@gen
fn input():
    let pool = ["A", "B", "C", "FOO", "BAR", "x_1", "y_2"]
    fn any() -> Str = pool[rand(0, pool.len - 1)]
    let count = rand(1, 40)
    var defined = Set<Str>()
    print(count)
    for _ in 0..count:
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
