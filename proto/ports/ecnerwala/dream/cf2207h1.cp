// Codeforces 2207 H1: recover a hidden min/max circuit over n leaves
// (interactive).
//
// `ask(x)` shows the circuit the permutation `x` and answers which position's
// value comes out. `solve(span, gate, m)` builds the subcircuit over `span`
// whose output position is `m`: leaves left of `m` are joined with the same
// gate, leaves right of it (shown reversed) with the other one. What it
// returns is the chain of subtrees whose spans tile `span` left to right; the
// two chains hanging off `m` are zipped by asking, with the outermost pair of
// spans reversed, whether the output moves left of `m`'s subtree.
struct Node:
    span: Range<Int>
    gate: ?(is_max: Bool, a: Int, b: Int)

@!multitest
let n: Int = read()
var x = [i for i in 0..n]
var nodes = [Node(i..i + 1, none) for i in 0..n]

fn ask() -> Int:
    print("?", [v + 1 for v in x])
    x.position(of: read::<Int>() - 1)!

fn join(a: Int, b: Int, is_max: Bool) -> Int = nodes.push_id(
    Node(nodes[a].span.start..nodes[b].span.end, some((is_max, a, b)))
)

fn solve(span: Range<Int>, is_max: Bool, known: ?Int = none) -> List<Int>:
    if span.len <= 1: return span.to_list()
    let m = known ?? ask()
    var left = with x[span.start..=m].rotated_left(1): solve(span.start..m, is_max)
    var right = with x[m..span.end].reversed: solve(m + 1..span.end, not is_max)
    right.reverse()
    var mid = m
    while let a = left.last and let b = right.last:
        let moved_left = with x[
            nodes[a].span.start..nodes[b].span.end
        ].reversed: ask() < nodes[mid].span.start
        if moved_left: mid = join(mid, right.pop()!, not is_max)
        else: mid = join(left.pop()!, mid, is_max)
    left + [right.rev.fold(mid, (m, b) => join(m, b, not is_max))]

fn eval(vals: List<Int>, v: Int) -> Int:
    guard let g = nodes[v].gate else: return vals[v]
    let (a, b) = (eval(vals, g.a), eval(vals, g.b))
    if g.is_max: max(a, b) else: min(a, b)

let loc_incr = ask()
let loc_decr = with x.reversed: ask()
let is_max = loc_incr >= loc_decr
if not is_max: x.reverse()
let top = solve(0..n, is_max, known: if is_max: loc_incr else: loc_decr)
let root = top.rev.reduce((b, a) => join(a, b, is_max))!
print("!")
for vals in read_until<List<Int>>(n, end: 0):
    print(eval(vals, root))
