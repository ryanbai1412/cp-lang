// Codeforces 2207 H3: Bowser's Castle (Hard Version).
//
// The hidden circuit is a binary tree of min/max gates over the leaves
// 0..n, in order. A probe feeds a permutation `x` and learns which leaf's
// value reaches the root.
//
// `solve(span)` assumes `x[span]` is sorted in favour of the gate kind
// `is_max` (ascending for max, descending for min). The winner `m` is the
// leaf whose path to the segment root only ever loses on the side `x`
// favours, so the leaves left of `m` attach by `is_max` gates and those
// right of it by the opposite kind:
//   - rotating `m` to the bottom of `span.start..=m` keeps `span.start..m` sorted for `is_max`,
//   - reversing `m..span.end` sorts `m + 1..span.end` for the opposite kind.
// Both sides come back as chains of subtrees hanging off `m`. Reversing `x`
// over the nearer left and right subtrees together makes the one that joins
// `m` first win the next probe, so one probe per join zips the chains.
let wire = Wire()

struct Gate:
    span: Range<Int>
    op: ?(is_max: Bool, a: Int, b: Int)

@!multitest
let n: 1..=5000 = read()
var x = [i + 1 for i in 0..n]
var gates = [Gate(i..i + 1, None) for i in 0..n]

fn probe() -> Int:
    print("?", x)
    let winner: Int = read()
    x.find(winner)!

fn join(a: Int, b: Int, is_max: Bool) -> Int = gates.push_id(
    Gate(gates[a].span.start..gates[b].span.end, Some((is_max, a, b)))
)

fn solve(span: Range<Int>, is_max: Bool, known_m: ?Int = None) -> List<Int>:
    if span.len <= 1: return span.collect()
    let m = known_m ?? probe()

    x[span.start..=m].rotate_left(1)
    var left = solve(span.start..m, is_max)
    x[span.start..=m].rotate_right(1)
    x[m..span.end].reverse()
    var right = solve(m + 1..span.end, not is_max)
    x[m..span.end].reverse()
    right.reverse()

    var core = m
    while let a = left.last and let b = right.last:
        let outer = gates[a].span.start..gates[b].span.end
        x[outer].reverse()
        let moved_left = probe() < gates[core].span.start
        x[outer].reverse()
        if moved_left: core = join(core, right.pop()!, not is_max)
        else: core = join(left.pop()!, core, is_max)
    left + [right.rev.fold_from(core, (c, b) => join(c, b, not is_max))]

let win_up = probe()
x.reverse()
let win_down = probe()
x.reverse()
let is_max = win_up >= win_down
if not is_max: x.reverse()

let chain = solve(0..n, is_max, known_m: if is_max: win_up else: win_down)
let root = chain.rev.reduce((b, a) => join(a, b, is_max))!
print("!")

loop:
    let first: Int = read()
    if first == 0: break
    let vals = [first] + [read<Int>() for _ in 1..n]
    fn eval(g: Int) -> Int:
        guard let op = gates[g].op else: return vals[g]
        let (a, b) = (eval(op.a), eval(op.b))
        if op.is_max: max(a, b) else: min(a, b)

    print(eval(root))
