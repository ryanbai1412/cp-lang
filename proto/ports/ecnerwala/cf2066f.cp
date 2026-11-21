@!multitest
struct Op(l: Int, r: Int, vals: List<Int>)

let (n, m): (1..=500, 1..=500) = read()
domain PosA = 0..n
domain PosB = 0..m
domain CutA = 0..=n
domain CutB = 0..=m
domain Flag = 0..=1

struct Prev(i: CutA, j: CutB, f: Flag)

let a: [PosA]-1_000_000..=1_000_000 = read()
let b: [PosB]-1_000_000..=1_000_000 = read()

var b_val: [CutB, CutB]Int = 0
for i in 0..m:
    var cur_sum = b[PosB(i)]
    var best_sum = b[PosB(i)]
    for j in i..m:
        if j > i:
            if cur_sum < 0: cur_sum = 0
            cur_sum += b[PosB(j)]
            best_sum max= cur_sum
        b_val[CutB(i), CutB(j + 1)] = best_sum

var ranges: List<(Int, Int)> = []
var range_ends: [PosA]?Int = none
var alive: [PosA]Bool = true
var valid_b_end: [PosB]Int = m
var ops: List<Op> = []
var solved = false

loop:
    var best_sum = 0
    var cur_st: ?Int = none
    var cur_sum = 0
    var best_l: ?Int = none
    var best_r = 0
    for i in 0..n:
        if alive[PosA(i)]:
            if cur_st == none or cur_sum < 0:
                cur_st = i
                cur_sum = 0
            cur_sum += a[PosA(i)]
            if best_l == none or cur_sum >= best_sum:
                best_l = cur_st
                best_r = i + 1
                best_sum = cur_sum
        else:
            cur_st = none
    let bl = best_l ?? break
    ranges.push((bl, best_r))
    for i in bl..best_r: alive[PosA(i)] = false
    range_ends[PosA(bl)] = best_r
    for i in 0..m:
        let pi = PosB(i)
        while valid_b_end[pi] > i and b_val[CutB(i), CutB(valid_b_end[pi])] > best_sum:
            valid_b_end[pi] -= 1

    var dp: [CutA, CutB, Flag]?Prev = none
    dp[CutA(0), CutB(0), Flag(0)] = Prev(CutA(0), CutB(0), Flag(0))
    var i = 0
    while i < n:
        if let ni = range_ends[PosA(i)]:
            var nxt0 = 0
            var nxt1 = 0
            for j in 0..m:
                nxt0 max= j + 1
                nxt1 max= j + 1
                let vend = valid_b_end[PosB(j)]
                if dp[CutA(i), CutB(j), Flag(0)] != none:
                    while nxt0 <= vend:
                        dp[CutA(ni), CutB(nxt0), Flag(0)] = Prev(CutA(i), CutB(j), Flag(0))
                        nxt0 += 1
                    while nxt1 <= m:
                        dp[CutA(ni), CutB(nxt1), Flag(1)] = Prev(CutA(i), CutB(j), Flag(0))
                        nxt1 += 1
                if dp[CutA(i), CutB(j), Flag(1)] != none:
                    while nxt1 <= vend:
                        dp[CutA(ni), CutB(nxt1), Flag(1)] = Prev(CutA(i), CutB(j), Flag(1))
                        nxt1 += 1
            i = ni
        else:
            for j in 0..m:
                if b[PosB(j)] == a[PosA(i)]:
                    if dp[CutA(i), CutB(j), Flag(0)] != none:
                        dp[CutA(i + 1), CutB(j + 1), Flag(0)] = Prev(CutA(i), CutB(j), Flag(0))
                    if dp[CutA(i), CutB(j), Flag(1)] != none:
                        dp[CutA(i + 1), CutB(j + 1), Flag(1)] = Prev(CutA(i), CutB(j), Flag(1))
            i += 1

    if dp[CutA(n), CutB(m), Flag(1)] != none:
        var repl: [PosA]List<Int> = []
        var cur = Prev(CutA(n), CutB(m), Flag(1))
        var final_op = Op(0, 0, [])
        while cur.i.to_idx > 0:
            let prv = dp[cur.i, cur.j, cur.f]!
            let slice = [b[PosB(k)] for k in prv.j.to_idx..cur.j.to_idx]
            if prv.f.to_idx == 0 and cur.f.to_idx == 1:
                repl[PosA(prv.i.to_idx)] = [best_sum]
                final_op = Op(prv.j.to_idx, prv.j.to_idx + 1, slice)
            else:
                repl[PosA(prv.i.to_idx)] = slice
            cur = prv
        var delta_sz: [PosA]Int = 0
        for (l, r) in ranges:
            var st = l
            for z in 0..l: st += delta_sz[PosA(z)]
            delta_sz[PosA(l)] = repl[PosA(l)].len - (r - l)
            ops.push(Op(st, st + (r - l), repl[PosA(l)]))
        ops.push(final_op)
        solved = true
        break

if solved:
    print(ops.len)
    for op in ops:
        print(op.l + 1, op.r, op.vals.len)
        print(op.vals)
else:
    print(-1)
