// Codeforces 2207 H3: Bowser's Castle (Hard Version).
//
// The hidden circuit is a binary tree of min/max gates over the leaves
// 0..n, in order. A probe feeds a permutation `x` and learns which leaf's
// value reaches the root.
//
// `solve(l..r)` assumes `x` on l..r is sorted in favour of the gate kind
// `is_max` (ascending for max, descending for min). The winner `m` is the
// leaf whose path to the segment root only ever loses on the side `x`
// favours, so the leaves left of `m` attach by `is_max` gates and those
// right of it by the opposite kind:
//   - rotating `m` to the bottom of l..=m keeps l..m sorted for `is_max`,
//   - reversing m..r sorts m+1..r for the opposite kind.
// Both sides come back as chains of subtrees hanging off `m`. Reversing `x`
// over the nearer left and right subtrees together makes the one that joins
// `m` first win the next probe, so one probe per join zips the chains.
@!interactive

struct Gate:
    l: Int
    r: Int
    is_max: Bool
    kids: ?(Int, Int)

@!multitest
let n: 1..=5000 = read()
var x = [i + 1 for i in 0..n]
var gates = [Gate(i, i + 1, false, none) for i in 0..n]

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
    gates.push(Gate(gates[a].l, gates[b].r, is_max, (a, b)))
    gates.len - 1

fn solve(l: Int, r: Int, is_max: Bool, known_m: ?Int) -> List<Int>:
    if r - l <= 1: return [i for i in l..r]
    let m = known_m ?? probe()

    x[l..=m].rotate_left(1)
    var left = solve(l, m, is_max, none)
    x[l..=m].rotate_right(1)

    x[m..r].reverse()
    var right = solve(m + 1, r, not is_max, none)
    x[m..r].reverse()
    right.reverse()

    var core = m
    while let a = left.last and let b = right.last:
        if probe_reversed(gates[a].l..gates[b].r) < gates[core].l:
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

loop:
    let first: Int = read()
    if first == 0: break
    let vals = [first] + [read<Int>() for _ in 1..n]

    fn eval(g: Int) -> Int:
        if let (a, b) = gates[g].kids:
            if gates[g].is_max: max(eval(a), eval(b)) else: min(eval(a), eval(b))
        else:
            vals[g]

    print(eval(root))
