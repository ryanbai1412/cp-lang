// Source: Codeforces 2207H1 (Bowser's Castle, easy),
//   https://codeforces.com/contest/2207/problem/H1
// Problem: interactive. A hidden min-max function f of x_1..x_n (only min and max calls, each
//   variable once, in order), n <= 70. Ask "? x_1 .. x_n" to get f (at most 1e4 asks in all),
//   print "!", then answer the judge's inputs with f until a line "0". Multitest, sum n <= 70.
// Solution: asking a permutation tells which position's value comes out. Comparing the output
//   of 1..n and its reverse gives the root gate. solve(span, gate, m) builds the subcircuit with
//   output position m: leaves left of m join with the gate, right of m (shown reversed) with the
//   other one, recursively; the two chains of subtrees hanging off m are zipped by asking, with
//   the outermost pair of spans reversed, whether the output moves left of m's subtree. About 2n
//   asks; each judge input is evaluated on the tree in O(n).
// Bounds: n: 1..=5000

struct Node:
    span: Range<Int>
    gate: ?(is_max: Bool, a: Int, b: Int)

@!multitest
let n: 1..=5000 = read()
var x = (0..n).collect()
var nodes = [Node(i..i + 1, None) for i in 0..n]

fn ask() -> Int:
    print("?", [v + 1 for v in x])
    x.find(read<Int>() - 1)!

fn cover(a: Int, b: Int) -> Range<Int> = nodes[a].span.start..nodes[b].span.end

fn join(a: Int, b: Int, is_max: Bool) -> Int:
    nodes.push(Node(cover(a, b), (is_max, a, b)))
    nodes.len - 1

fn solve(l: Int, r: Int, is_max: Bool, known: ?Int = None) -> List<Int>:
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
