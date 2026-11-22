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
var nodes = [Node(i..i + 1, none) for i in 0..n]

fn ask() -> Int:
    print("?", [v + 1 for v in x])
    x.find(read<Int>() - 1)!

fn cover(a: Int, b: Int) -> Range<Int> = nodes[a].span.start..nodes[b].span.end

fn join(a: Int, b: Int, is_max: Bool) -> Int:
    nodes.push(Node(cover(a, b), (is_max, a, b)))
    nodes.len - 1

fn solve(l: Int, r: Int, is_max: Bool, known: ?Int = none) -> List<Int>:
    if r - l <= 1: return [l for _ in l..r]
    let m = known ?? ask()
    x[l..=m].rotate_left(1)
    var left = solve(l, m, is_max)
    x[l..=m].rotate_right(1)
    x[m..r].reverse()
    var right = solve(m + 1, r, not is_max)
    x[m..r].reverse()
    right.reverse()
    var mid = m
    while let a = left.last and let b = right.last:
        let outer = cover(a, b)
        x[outer].reverse()
        let moved_left = ask() < nodes[mid].span.start
        x[outer].reverse()
        if moved_left: mid = join(mid, right.pop()!, not is_max)
        else: mid = join(left.pop()!, mid, is_max)
    while let b = right.pop(): mid = join(mid, b, not is_max)
    left + [mid]

fn eval(vals: List<Int>, v: Int) -> Int:
    if let g = nodes[v].gate:
        let (a, b) = (eval(vals, g.a), eval(vals, g.b))
        if g.is_max: max(a, b) else: min(a, b)
    else: vals[v]

let loc_incr = ask()
x.reverse()
let loc_decr = ask()
let is_max = loc_incr >= loc_decr
if is_max: x.reverse()
var top = solve(0, n, is_max, if is_max: loc_incr else: loc_decr)
var root = top.pop()!
while let a = top.pop(): root = join(a, root, is_max)
print("!")
loop:
    let first_v: Int = read()
    if first_v == 0: break
    let vals = [first_v] + [read<Int>() for _ in 1..n]
    print(eval(vals, root))
