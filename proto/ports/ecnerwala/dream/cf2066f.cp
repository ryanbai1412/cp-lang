// Greedily extract the maximum-sum subarray of still-unreplaced elements of a
// (giving value v). Each extracted block can expand into any subarray of b with
// max-subarray-sum <= v; one chosen block may instead hold [v] and expand into
// any subarray of b as the final operation. DP checks if a splits into b.
@!multitest
domain PosA = read(1..=500)
domain PosB = read(1..=500)
let a: [PosA]-1e6..=1e6 = read()
let b: [PosB]-1e6..=1e6 = read()

struct Op(l: Int, r: Int, vals: List<Int>)
enum Phase: Before, After
struct Prev(i: PosA.Cut, j: PosB.Cut, phase: Phase)

// max_sub[l, r] = maximum non-empty subarray sum of b[l..r].
let max_sub = [PosB.Cut, PosB.Cut] (l, r) => b[l..r].max_subarray_sum()

var alive: [PosA]Bool = true
var block_end: [PosA]?PosA.Cut = none
var blocks: List<(PosA, PosA.Cut)> = []
var valid_end: [PosB]PosB.Cut = PosB.Cut.last

fn next_block() -> ?(PosA, PosA.Cut, Int):
    // Maximum-sum contiguous subarray among alive positions of a.
    var best: ?(PosA, PosA.Cut, Int) = none
    var cur_start: ?PosA = none
    var cur_sum = 0
    for i in PosA:
        if not alive[i]: cur_start = none; continue
        if cur_start == none or cur_sum < 0: (cur_start, cur_sum) = (i, 0)
        cur_sum += a[i]
        if best == none or cur_sum >= best!.2: best = (cur_start!, i.r, cur_sum)
    best

fn try_match(best_sum: Int) -> ?List<Op>:
    var dp: [PosA.Cut, PosB.Cut, Phase]?Prev = none
    dp[PosA.Cut.first, PosB.Cut.first, Phase.Before] = Prev(PosA.Cut.first, PosB.Cut.first, Phase.Before)
    var i = PosA.Cut.first
    while i < PosA.Cut.last:
        if let ni = block_end[i.after]:
            var (nxt0, nxt1) = (PosB.Cut.first, PosB.Cut.first)
            for j in PosB:
                nxt0 max= j.r
                nxt1 max= j.r
                let vend = valid_end[j]
                if dp[i, j.l, Phase.Before] != none:
                    for c in nxt0..=vend: dp[ni, c, Phase.Before] = Prev(i, j.l, Phase.Before)
                    nxt0 max= vend.next
                    for c in nxt1..=PosB.Cut.last: dp[ni, c, Phase.After] = Prev(i, j.l, Phase.Before)
                    nxt1 = PosB.Cut.last
                if dp[i, j.l, Phase.After] != none:
                    for c in nxt1..=vend: dp[ni, c, Phase.After] = Prev(i, j.l, Phase.After)
                    nxt1 max= vend.next
            i = ni
        else:
            for j in PosB where b[j] == a[i.after]:
                for ph in Phase where dp[i, j.l, ph] != none:
                    dp[i.next, j.r, ph] = Prev(i, j.l, ph)
            i = i.next

    if dp[PosA.Cut.last, PosB.Cut.last, Phase.After] == none: return none

    var repl: [PosA]List<Int> = []
    var cur = Prev(PosA.Cut.last, PosB.Cut.last, Phase.After)
    var final_op = Op(0, 0, [])
    while cur.i > PosA.Cut.first:
        let prv = dp[cur.i, cur.j, cur.phase]!
        let slice = b[prv.j..cur.j].list()
        if prv.phase == Phase.Before and cur.phase == Phase.After:
            repl[prv.i.after] = [best_sum]
            final_op = Op(prv.j.ord, prv.j.ord + 1, slice)
        else:
            repl[prv.i.after] = slice
        cur = prv

    var delta: [PosA]Int = 0
    var ops: List<Op> = []
    for (l, r) in blocks:
        let st = l.ord + sum(delta[z] for z in PosA.first..l)
        delta[l] = repl[l].len - (r - l)
        ops.push(Op(st, st + (r - l), repl[l]))
    ops.push(final_op)
    ops

while let (bl, br, best_sum) = next_block():
    blocks.push((bl, br))
    for i in bl..br: alive[i] = false
    block_end[bl] = br
    for j in PosB:
        while valid_end[j] > j.l and max_sub[j.l, valid_end[j]] > best_sum:
            valid_end[j] = valid_end[j].prev
    if let ops = try_match(best_sum):
        print(ops.len)
        for op in ops:
            print(op.l + 1, op.r, op.vals.len)
            print(op.vals)
        return

print(-1)
