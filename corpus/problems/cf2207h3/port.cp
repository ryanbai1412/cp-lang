// Source: Codeforces 2207H3 (Bowser's Castle, Hard Version),
//   https://codeforces.com/contest/2207/problem/H3
// limits: 8000 ms, 250 MB
// Problem: interactive, multitest. A hidden min-max function f of x_1..x_n (min/max calls, each
//   variable once, in order) is a binary tree of min/max gates over the leaves. Ask f(x) with at
//   most 1500 queries in total, then answer up to 5000 of the judge's f(x). Sum of n <= 400.
// Solution: probe with permutations, so the returned value names the winning leaf. solve(l..r)
//   keeps x on l..r sorted in favour of one gate kind; its winner m splits it: leaves left of m
//   hang off m by that kind, leaves right of it by the other (rotate l..=m, reverse m..r and
//   recurse). One probe with the nearer left and right subtrees reversed tells which joins m
//   first, so the two chains zip with one probe per join. Then evaluate the built tree per input.
// Bounds: n: 1..=5000
let wire = Wire()

struct Gate:
    span: Range<Int>
    is_max: Bool
    kids: ?(Int, Int)

@!multitest
let n: 1..=5000 = read()
var x = [i + 1 for i in 0..n]
var gates = [Gate(i..i + 1, false, None) for i in 0..n]

fn probe() -> Int:
    print("?", x)
    let winner: Int = read()
    x.find(winner)!

fn probe_reversed(span: Range<Int>) -> Int:
    x[span].reverse()
    let winner = probe()
    x[span].reverse()
    winner

fn join(a: Int, b: Int, is_max: Bool) -> Int:
    gates.push(Gate(gates[a].span.start..gates[b].span.end, is_max, (a, b)))
    gates.len - 1

fn solve(l: Int, r: Int, is_max: Bool, known_m: ?Int) -> List<Int>:
    if r - l <= 1: return (l..r).collect()
    let m = known_m ?? probe()

    x[l..=m].rotate_left(1)
    var left = solve(l, m, is_max, None)
    x[l..=m].rotate_right(1)

    x[m..r].reverse()
    var right = solve(m + 1, r, not is_max, None)
    x[m..r].reverse()
    right.reverse()

    var core = m
    while let a = left.last and let b = right.last:
        if probe_reversed(gates[a].span.start..gates[b].span.end) < gates[core].span.start:
            core = join(core, right.pop()!, not is_max)
        else:
            core = join(left.pop()!, core, is_max)
    while let b = right.pop(): core = join(core, b, not is_max)
    left.push(core)
    left

let win_up = probe()
let win_down = probe_reversed(0..n)
let is_max = win_up >= win_down
if not is_max: x.reverse()

var chain = solve(0, n, is_max, if is_max: win_up else: win_down)
var root = chain.pop()!
while let a = chain.pop(): root = join(a, root, is_max)
print("!")

while true:
    let first: Int = read()
    if first == 0: break
    let vals = [first] + [read<Int>() for _ in 1..n]

    fn eval(g: Int) -> Int:
        if let (a, b) = gates[g].kids:
            if gates[g].is_max: max(eval(a), eval(b)) else: min(eval(a), eval(b))
        else:
            vals[g]

    print(eval(root))
