// Greedily extract the maximum-sum subarray of still-unreplaced elements of a
// (giving value v). Each extracted block can expand into any subarray of b with
// max-subarray-sum <= v; one chosen block may instead hold [v] and expand into
// any subarray of b as the final operation. DP checks if a splits into b.
@!multitest
struct Op(l: Int, r: Int, vals: List<Int>)
struct Prev(i: Int, j: Int, after: Bool)

let (n, m): (1..=500, 1..=500) = read()
domain PosA = 0..n
domain PosB = 0..m
domain CutA = 0..=n
domain CutB = 0..=m

type Val = -1_000_000..=1_000_000
let a: [PosA]Val = read()
let b: [PosB]Val = read()

var max_sub: [CutB, CutB]Int = 0
for i in 0..m:
    var cur_sum = b[PosB(i)]
    var best_sum = b[PosB(i)]
    for j in i..m:
        if j > i:
            if cur_sum < 0: cur_sum = 0
            cur_sum += b[PosB(j)]
            best_sum max= cur_sum
        max_sub[CutB(i), CutB(j + 1)] = best_sum

var alive: [PosA]Bool = true
var block_end: [PosA]?Int = none
var blocks: List<(Int, Int)> = []
var valid_end: [PosB]Int = m

fn next_block() -> ?(Int, Int, Int):
    var best: ?(Int, Int, Int) = none
    var cur_start: ?Int = none
    var cur_sum = 0
    for i in 0..n:
        if not alive[PosA(i)]:
            cur_start = none
            continue
        if cur_start == none or cur_sum < 0:
            cur_start = i
            cur_sum = 0
        cur_sum += a[PosA(i)]
        if best == none or cur_sum >= best!.2:
            best = (cur_start!, i + 1, cur_sum)
    best

fn try_match(best_sum: Int) -> ?List<Op>:
    var dp0: [CutA, CutB]?Prev = none
    var dp1: [CutA, CutB]?Prev = none
    dp0[CutA(0), CutB(0)] = Prev(0, 0, false)
    var i = 0
    while i < n:
        if let ni = block_end[PosA(i)]:
            var nxt0 = 0
            var nxt1 = 0
            for j in 0..m:
                nxt0 max= j + 1
                nxt1 max= j + 1
                let vend = valid_end[PosB(j)]
                if dp0[CutA(i), CutB(j)] != none:
                    while nxt0 <= vend:
                        dp0[CutA(ni), CutB(nxt0)] = Prev(i, j, false)
                        nxt0 += 1
                    while nxt1 <= m:
                        dp1[CutA(ni), CutB(nxt1)] = Prev(i, j, false)
                        nxt1 += 1
                if dp1[CutA(i), CutB(j)] != none:
                    while nxt1 <= vend:
                        dp1[CutA(ni), CutB(nxt1)] = Prev(i, j, true)
                        nxt1 += 1
            i = ni
        else:
            for j in 0..m if b[PosB(j)] == a[PosA(i)]:
                if dp0[CutA(i), CutB(j)] != none:
                    dp0[CutA(i + 1), CutB(j + 1)] = Prev(i, j, false)
                if dp1[CutA(i), CutB(j)] != none:
                    dp1[CutA(i + 1), CutB(j + 1)] = Prev(i, j, true)
            i += 1

    if dp1[CutA(n), CutB(m)] == none: return none

    var repl: [PosA]List<Int> = []
    var cur = Prev(n, m, true)
    var final_op = Op(0, 0, [])
    while cur.i > 0:
        let prv = if cur.after: dp1[CutA(cur.i), CutB(cur.j)]! else: dp0[CutA(cur.i), CutB(cur.j)]!
        let slice = [b[PosB(k)] for k in prv.j..cur.j]
        if not prv.after and cur.after:
            repl[PosA(prv.i)] = [best_sum]
            final_op = Op(prv.j, prv.j + 1, slice)
        else:
            repl[PosA(prv.i)] = slice
        cur = prv

    var delta: [PosA]Int = 0
    var ops: List<Op> = []
    for (l, r) in blocks:
        let st = l + sum(delta[PosA(z)] for z in 0..l)
        delta[PosA(l)] = repl[PosA(l)].len - (r - l)
        ops.push(Op(st, st + (r - l), repl[PosA(l)]))
    ops.push(final_op)
    ops

fn solve() -> ?List<Op>:
    while let (bl, br, best_sum) = next_block():
        blocks.push((bl, br))
        for i in bl..br: alive[PosA(i)] = false
        block_end[PosA(bl)] = br
        for i in 0..m:
            let pi = PosB(i)
            while valid_end[pi] > i and max_sub[CutB(i), CutB(valid_end[pi])] > best_sum:
                valid_end[pi] -= 1
        if let ops = try_match(best_sum): return ops
    none

if let ops = solve():
    print(ops.len)
    for op in ops:
        print(op.l + 1, op.r, op.vals.len)
        print(op.vals)
else:
    print(-1)
