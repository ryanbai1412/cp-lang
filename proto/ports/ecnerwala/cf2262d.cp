// Codeforces 2262 D: a token sits on some cell; Inc (1) increments a[cur],
// Jump (2) moves to cell a[cur]. Turn a into b or report -1.
// Only cells whose target chain b[i] -> b[b[i]] -> .. reaches the last
// differing cell may change. Scanning right to left, `via[i]` tracks the phase
// of cell i: detour to an earlier unfinished cell t != i, sweep unfinished
// cells below i (`via[i] == i`), or finish up to b[i] (`via[i] == none`).
enum Op: Inc | Jump

fn op_code(op: Op) -> Int = match op:
    Op.Inc => 1
    Op.Jump => 2

@!multitest
let n: 1..=2000 = read()
domain Cell = 1..=n
var a: [Cell]Cell = read()
let b: [Cell]Cell = read()

fn plan() -> ?(start: Cell, ops: List<Op>):
    if any(a[i] > b[i] for i in Cell): return none
    let last = Cell.rev.first(i => a[i] != b[i]) ?? Cell.first

    var bound = last
    var okay: [Cell]Bool = false
    okay[last] = true
    var via: [Cell]?Cell = none
    var low = last
    for i in (Cell.first..last).rev:
        okay[i] = b[i] > i and b[i] <= bound and okay[b[i]]
        if a[i] == b[i]: continue
        if not okay[i]: return none
        if i < a[low]:
            bound = i
            via[low] = low
            low = i
        elif a[i] < a[low]:
            via[low] = i
            low = i
    via[low] = low

    fn advance_phase(cur: Cell):
        if let t = via[cur], t != cur and via[t] == none: via[cur] = cur
        if via[cur] == cur and a[cur] >= cur: via[cur] = none

    fn next_op(cur: Cell) -> Op:
        if let t = via[cur]:
            if t != cur: return if a[cur] == t: Op.Jump else: Op.Inc
            return if a[a[cur]] != b[a[cur]]: Op.Jump else: Op.Inc
        if a[cur] == b[cur]: Op.Jump else: Op.Inc

    var ops: List<Op> = []
    var cur = low
    while a[last] != b[last]:
        advance_phase(cur)
        let op = next_op(cur)
        ops.push(op)
        match op:
            Op.Jump => cur = a[cur]
            Op.Inc => a[cur] = (a[cur] + 1)!
    (low, ops)

if let (start, ops) = plan():
    print(ops.len, start)
    print([op_code(op) for op in ops])
else:
    print(-1)
