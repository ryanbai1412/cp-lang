// `?Int` is one `Int` everywhere, with -2^63 as `none`: through fns,
// tuples, records, maps, lists and printing, every edge value reads
// back as the interpreter has it.
fn pick(a: ?Int, b: ?Int) -> ?Int: a ?? b
fn lo(xs: List<?Int>) -> ?Int:
    var best: ?Int = none
    for x in xs: best min= x
    best
let edges: List<?Int> = [none, Int.MIN, -1, 0, Int.MAX, none]
var xs = edges
xs.push(xs[1])
print(xs, xs.len)
print(pick(none, Int.MIN) ?? 0, pick(Int.MAX, none) ?? 0, pick(none, none) == none)
print(lo(edges) ?? 0, lo([]) == none)
let pair: (?Int, Int) = (Int.MIN, 7)
let rec: (at: ?Int, n: ?Int) = (at: none, n: Int.MAX)
print(pair.0 ?? 0, pair.1, rec.at == none, rec.n ?? 0)
var m: Map<Int, ?Int> = []
m[1] = Int.MIN
m[2] = none
print(m[1] ?? 0, m[2] == none, m.get(3) == none)
var held: ?Int = Int.MIN
if let h = held: held = h + 1
print(held ?? 0, held == Int.MIN + 1, held != none)
held = none
print(held == none, held ?? Int.MIN)
