// Source: Codeforces 2262D (PLUSworld), https://codeforces.com/contest/2262/problem/D
// limits: 2000 ms, 256 MB
// Problem: barn i has a road to a[i]; target roads b[i] >= i. Pick a start barn p, then do ops
//   1 (a[p] += 1, only if a[p] < n) and 2 (p = a[p]) until a == b, at most 2n^2 ops; print the
//   op count, p and the ops, or -1. Multitest, n <= 1000, sum of n^2 <= 1e6.
// Solution: a[i] <= b[i] is needed, and a finished cell is left along b[i], so every differing
//   cell before the last one must reach it by a strictly increasing chain of b inside `bound`.
//   Scanning right to left from the last differing cell keeps the leftmost anchor `low`: a cell
//   below a[low] must be done first (low gets Sweep, bound shrinks), a cell with a[i] < a[low] is
//   a Detour of low. Then simulate from low: each cell goes Detour(t) -> Sweep -> Finish, choosing
//   Inc or Jump from its phase; O(n^2) ops and time.
// Bounds: n: 1..=2000; a, b: 1..=n
enum Op: Inc = 1, Jump = 2

@!multitest
let n: 1..=2000 = read()
domain Cell = 1..=n
var a: [Cell]Cell = read()
let b: [Cell]Cell = read()

enum Phase:
    Detour(Cell)
    Sweep
    Finish

fn plan() -> ?(start: Cell, ops: List<Op>):
    if Cell.any(i => a[i] > b[i]): return None
    let last = Cell.rev.find(i => a[i] != b[i]) ?? Cell.first!

    var bound = last
    var okay: [Cell]Bool = false
    okay[last] = true
    var phase: [Cell]Phase = Phase.Finish
    var low = last
    for i in (Cell.first!..last).rev:
        okay[i] = b[i] > i and b[i] <= bound and okay[b[i]]
        if a[i] == b[i]: continue
        if not okay[i]: return None
        if i < a[low]:
            bound = i
            phase[low] = Phase.Sweep
            low = i
        elif a[i] < a[low]:
            phase[low] = Phase.Detour(i)
            low = i
    phase[low] = Phase.Sweep

    fn is_finish(c: Cell) -> Bool = match phase[c]:
        Phase.Finish => true
        _ => false

    fn advance_phase(cur: Cell):
        phase[cur] = match phase[cur]:
            Phase.Detour(t) =>
                if is_finish(t):
                    if a[cur] >= cur: Phase.Finish else: Phase.Sweep
                else:
                    Phase.Detour(t)
            Phase.Sweep => if a[cur] >= cur: Phase.Finish else: Phase.Sweep
            Phase.Finish => Phase.Finish

    fn next_op(cur: Cell) -> Op = match phase[cur]:
        Phase.Detour(t) => if a[cur] == t: Op.Jump else: Op.Inc
        Phase.Sweep => if a[a[cur]] != b[a[cur]]: Op.Jump else: Op.Inc
        Phase.Finish => if a[cur] == b[cur]: Op.Jump else: Op.Inc

    var ops: List<Op> = []
    var cur = low
    while a[last] != b[last]:
        advance_phase(cur)
        let op = next_op(cur)
        ops.push(op)
        match op:
            Op.Jump => cur = a[cur]
            Op.Inc => a[cur] = a[cur].offset(1)!
    (low, ops)

if let (start, ops) = plan():
    print(ops.len, start)
    print([Int(op) for op in ops])
else:
    print(-1)
