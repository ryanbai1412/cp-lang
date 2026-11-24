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
let n: 1..=5000 = read()
var x = [i for i in 0..n]
var nodes = [Node(i..i + 1, None) for i in 0..n]

fn ask() -> Int:
    print("?", [v + 1 for v in x])
    x.find(read<Int>() - 1)!

fn join(a: Int, b: Int, is_max: Bool) -> Int = nodes.push_id(
    Node(nodes[a].span.start..nodes[b].span.end, Some((is_max, a, b)))
)

fn solve(span: Range<Int>, is_max: Bool, known: ?Int = None) -> List<Int>:
    if span.len <= 1: return span.collect()
    let m = known ?? ask()
    x[span.start..=m].rotate_left(1)
    var left = solve(span.start..m, is_max)
    x[span.start..=m].rotate_right(1)
    x[m..span.end].reverse()
    var right = solve(m + 1..span.end, not is_max)
    x[m..span.end].reverse()
    right.reverse()
    var mid = m
    while let a = left.last and let b = right.last:
        let outer = nodes[a].span.start..nodes[b].span.end
        x[outer].reverse()
        let moved_left = ask() < nodes[mid].span.start
        x[outer].reverse()
        if moved_left: mid = join(mid, right.pop()!, not is_max)
        else: mid = join(left.pop()!, mid, is_max)
    left + [right.rev.fold((m, b) => join(m, b, not is_max), mid)]

fn eval(vals: List<Int>, v: Int) -> Int:
    guard let g = nodes[v].gate else: return vals[v]
    let (a, b) = (eval(vals, g.a), eval(vals, g.b))
    if g.is_max: max(a, b) else: min(a, b)

let loc_incr = ask()
x.reverse()
let loc_decr = ask()
x.reverse()
let is_max = loc_incr >= loc_decr
if not is_max: x.reverse()
let top = solve(0..n, is_max, known: if is_max: loc_incr else: loc_decr)
let root = top.rev.reduce((b, a) => join(a, b, is_max))!
print("!")
loop:
    let first_v: Int = read()
    if first_v == 0: break
    let vals = [first_v] + [read<Int>() for _ in 1..n]
    print(eval(vals, root))
