// Codeforces 2262 D: a token sits on some cell; Inc (1) increments a[cur],
// Jump (2) moves to cell a[cur]. Turn `a` into `b` or report -1.
//
// Reachability & phase invariant:
// - `Inc` only increases `a[cur]`, so `a[i] <= b[i]` is necessary. Once a cell
//   finishes (`a[i] == b[i]`), the token exits to `b[i]`, so every differing
//   cell `i < last` must have a strictly increasing target chain
//   `i -> b[i] -> ... -> last` staying `<= bound` (`okay[i]`).
// - Scanning right-to-left from `last` maintains `low` (the current leftmost
//   anchor) and `bound` (the leftmost cell no earlier `b[i]` may jump past):
//   - If `i < a[low]`, `low` cannot reach `i` by incrementing `a[low]`, so `i`
//     must run before `low`: `low` enters `Sweep`, `bound` tightens to `i`,
//     and `low` becomes `i`.
//   - Else if `a[i] < a[low]`, `low` can reach `i` while incrementing `a[low]`,
//     so `low` records `Detour(i)` (jump to `i` when `a[low] == i`) and `low`
//     becomes `i`.
// - Starting at `low` in `Sweep` mode, each cell moves through
//   `Detour(t) -> Sweep -> Finish`: first jumping to `t` until `t` is `Finish`,
//   then jumping to any unfinished cell `a[cur] < cur`, and finally raising
//   `a[cur]` to `b[cur]` and exiting along `b[cur]`.
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
    if any(a[i] > b[i] for i in Cell): return None
    let last = Cell.rev.first(i => a[i] != b[i]) ?? Cell.first!

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
            Op.Inc => a[cur] = a[cur].next!
    (low, ops)

if let (start, ops) = plan():
    print(ops.len, start)
    print([Int(op) for op in ops])
else:
    print(-1)
