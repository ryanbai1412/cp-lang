// Runnable CP Lang v2.2 port of noi04p3 (NOI '04 P3 - Manhattan)
// Original C++: corpus/submissions/5032868.cpp
// Runs directly under the Rust `cpl run` and `cpl build` (c521b2c).
// Exercises v2.2 tropical `?Int` (`+` absorbs `none`, `min=` skips `none`, `??`),
// eliminating the original C++'s `1e9` sentinel completely.
// Workarounds vs blind first draft (noi04p3_draft.cp):
// - No `for mask in Subset<Row>:` or `for p in ?Col:` in Rust syntax -> explicit bitmask & encoded state
// - No `for d in VDir if cond:` filter -> explicit `if`

struct Query:
    x1: Int
    y1: Int
    x2: Int
    y2: Int

fn encrypt(l0: Int, l1: Int) -> Int:
    if l0 > l1:
        return (l1 + 1) << 1
    ((l0 + 1) << 1) | 1

let n: Int = read()
let m: Int = read()
let sh: Str = read()
let sv: Str = read()

var hori: List<Int> = []
for i in 0..n:
    hori.push(if sh[i] == 'E': 1 else: 0)

var vert: List<Int> = []
for i in 0..m:
    vert.push(if sv[i] == 'S': 1 else: 0)

var costh: List<Int> = []
for _ in 0..n:
    let c: Int = read()
    costh.push(c)

var costv: List<Int> = []
for _ in 0..m:
    let c: Int = read()
    costv.push(c)

let q: Int = read()
var queries: List<List<Query>> = []
for _ in 0..m:
    var empty: List<Query> = []
    queries.push(empty)

for _ in 0..q:
    let x1_in: Int = read()
    let y1_in: Int = read()
    let x2_in: Int = read()
    let y2_in: Int = read()
    let x1 = x1_in - 1
    let y1 = y1_in - 1
    let x2 = x2_in - 1
    let y2 = y2_in - 1
    if x1 != x2 or y1 != y2:
        let qr = Query(x1: x1, y1: y1, x2: x2, y2: y2)
        queries[y1].push(qr)
        if y1 != y2:
            queries[y2].push(qr)

let states = (m + 1) * 2
var best_ans: ?Int = none
var best_ah: Str = ""
var best_av: Str = ""

let num_masks = 1 << n
for bni in 0..num_masks:
    var dp: List<?Int> = []
    var nxt_dir: List<Int> = []
    var nxt_enc: List<Int> = []
    for _ in 0..((m + 1) * states):
        dp.push(none)
        nxt_dir.push(0)
        nxt_enc.push(0)
    for s in 0..states:
        dp[m * states + s] = 0

    for ind in (0..m).rev:
        let max_lind = max(1, ind)
        for ltype in 0..2:
            for lind in 0..max_lind:
                let enc = (lind << 1) | ltype
                var last0 = -1
                var last1 = -1
                if ind > 0:
                    if ltype == 0:
                        last0 = ind - 1
                        last1 = lind - 1
                    else:
                        last1 = ind - 1
                        last0 = lind - 1

                var poss0 = true
                var poss1 = true
                var aborted = false
                for qr in queries[ind]:
                    let x1 = qr.x1
                    let y1 = qr.y1
                    let x2 = qr.x2
                    let y2 = qr.y2
                    let left = if y1 > y2: 1 else: 0
                    let down = if x1 < x2: 1 else: 0
                    let bx1 = (bni >> x1) & 1
                    let bx2 = (bni >> x2) & 1
                    if x1 == x2:
                        if (bx1 ^ left) == 0:
                            aborted = true
                            break
                    elif y1 == y2:
                        if down == 0:
                            poss1 = false
                        else:
                            poss0 = false
                    else:
                        let ok1 = (bx1 ^ left) == 1
                        let ok2 = (bx2 ^ left) == 1
                        if ok1 and ok2:
                            let last_d = if down == 0: last0 else: last1
                            if last_d < min(y1, y2) and ind == max(y1, y2):
                                if down == 0:
                                    poss1 = false
                                else:
                                    poss0 = false
                        elif ok1 and not ok2:
                            if ind == y2:
                                if down == 0:
                                    poss1 = false
                                else:
                                    poss0 = false
                        elif not ok1 and ok2:
                            if ind == y1:
                                if down == 0:
                                    poss1 = false
                                else:
                                    poss0 = false
                        else:
                            var found_mid = false
                            let lo_x = min(x1, x2) + 1
                            let hi_x = max(x1, x2)
                            for mid_x in lo_x..hi_x:
                                if (((bni >> mid_x) & 1) ^ left) == 1:
                                    found_mid = true
                                    break
                            if not found_mid:
                                aborted = true
                                break
                            if down == 0:
                                poss1 = false
                            else:
                                poss0 = false

                if aborted:
                    continue

                var cur: ?Int = none
                var best_d = 0
                var best_e = 0
                for d in 0..2:
                    let allowed = if d == 0: poss0 else: poss1
                    if allowed:
                        let nl0 = if d == 0: ind else: last0
                        let nl1 = if d == 1: ind else: last1
                        let step = if d != vert[ind]: costv[ind] else: 0
                        let nenc = encrypt(nl0, nl1)
                        let cand = dp[(ind + 1) * states + nenc] + step
                        if cand != none:
                            if cur == none or (cand ?? 0) < (cur ?? 0):
                                cur = cand
                                best_d = d
                                best_e = nenc
                let slot = ind * states + enc
                dp[slot] = cur
                nxt_dir[slot] = best_d
                nxt_enc[slot] = best_e

    var hcost = 0
    for i in 0..n:
        if (((bni >> i) & 1) ^ hori[i]) == 1:
            hcost += costh[i]
    let total = dp[0] + hcost
    if total != none:
        if best_ans == none or (total ?? 0) < (best_ans ?? 0):
            best_ans = total
            var ah: Str = ""
            for i in 0..n:
                ah.push(if ((bni >> i) & 1) == 1: 'E' else: 'W')
            var av: Str = ""
            var j = 0
            for i in 0..m:
                let slot = i * states + j
                let d = nxt_dir[slot]
                av.push(if d == 1: 'S' else: 'N')
                j = nxt_enc[slot]
            best_ah = ah
            best_av = av

if best_ans == none:
    print("impossible")
else:
    print("possible")
    print(best_ans ?? 0)
    print(best_ah)
    print(best_av)
