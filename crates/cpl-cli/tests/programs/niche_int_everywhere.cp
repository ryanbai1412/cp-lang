// `?Int` is one `Int` everywhere, with -2^63 as `none`: through fns,
// tuples, records, maps, lists and printing, every edge value reads
// back as the interpreter has it.
fn pick(a: ?Int, b: ?Int) -> ?Int: a ?? b
fn lo(xs: List<?Int>) -> ?Int:
    var best: ?Int = None
    for x in xs: best min= x
    best
let edges: List<?Int> = [None, Int.MIN, -1, 0, Int.MAX, None]
var xs = edges.copy()
xs.push(xs[1])
print(xs, xs.len)
print(pick(None, Int.MIN) ?? 0, pick(Int.MAX, None) ?? 0, yes(pick(None, None) == None))
print(lo(edges) ?? 0, yes(lo([]) == None))
let pair: (?Int, Int) = (Int.MIN, 7)
let rec: (at: ?Int, n: ?Int) = (at: None, n: Int.MAX)
print(pair.0 ?? 0, pair.1, yes(rec.at == None), rec.n ?? 0)
var m: Map<Int, ?Int> = []
m[1] = Int.MIN
m[2] = None
print(m[1] ?? 0, yes(m[2] == None), yes(m.get(3) == None))
var held: ?Int = Int.MIN
if let h = held: held = h + 1
print(held ?? 0, yes(held == Int.MIN + 1), yes(held != None))
held = None
print(yes(held == None), held ?? Int.MIN)
