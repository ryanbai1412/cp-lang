// Problem: noi04p3 (NOI 2004 P3 - Manhattan)
// Original: corpus/submissions/5032868.cpp
// Blind Critic #5 — First Draft (strictly from docs/one-page-cheat-sheet.md v2.2)

enum VDir = N | S

let (n, m): (1..=10, 1..=100) = read()
domain Row = 1..=n
domain Col = 1..=m

let sh_str: Str = read()
let sv_str: Str = read()
let hori: [Row]Bool = [Row] r => sh_str[r.to_idx] == 'E'
let vert: [Col]VDir = [Col] c => if sv_str[c.to_idx] == 'S': S else: N
let costh: [Row]Int = read()
let costv: [Col]Int = read()

type Query = (x1: Row, y1: Col, x2: Row, y2: Col)
let q: Int = read()
var queries: [Col]List<Query> = []
for _ in 0..q:
    let qr: Query = read()
    if qr.x1 != qr.x2 or qr.y1 != qr.y2:
        queries[qr.y1].push(qr)
        if qr.y1 != qr.y2:
            queries[qr.y2].push(qr)

var best_cost: ?Int = none
var best_h = ""
var best_v = ""

fn other(d: VDir) -> VDir:
    match d:
        N => S
        S => N

for mask in Subset<Row>:
    var dp: [Col.cuts, ?Col, VDir]?Int = none
    var nxt: [Col, ?Col, VDir](VDir, ?Col, VDir) = (N, none, N)

    for p in ?Col:
        for d in VDir:
            dp[Col.last.r, p, d] = 0

    for c in (Col.first..=Col.last).rev:
        let prev_c = c - 1
        for ltype in VDir:
            let prev_opts: List<?Col> = if prev_c == none: [none] else: [none] + [p for p in ..prev_c!]
            'state: for prev_other in prev_opts:
                var last: [VDir]?Col = none
                last[ltype] = prev_c
                last[other(ltype)] = prev_other

                var poss: [VDir]Bool = true
                for qr in queries[c]:
                    let go_left = qr.y1 > qr.y2
                    let need_dir = if qr.x1 < qr.x2: S else: N
                    let r1_ok = (qr.x1 in mask) != go_left
                    let r2_ok = (qr.x2 in mask) != go_left
                    if qr.x1 == qr.x2:
                        if not r1_ok:
                            continue 'state
                        continue
                    if qr.y1 == qr.y2:
                        poss[other(need_dir)] = false
                        continue
                    if r1_ok and r2_ok:
                        let lo_y = min(qr.y1, qr.y2)
                        let hi_y = max(qr.y1, qr.y2)
                        let prev_before_lo = if let lc = last[need_dir]: lc < lo_y else: true
                        if prev_before_lo and c == hi_y:
                            poss[other(need_dir)] = false
                    elif r1_ok and not r2_ok:
                        if c == qr.y2:
                            poss[other(need_dir)] = false
                    elif not r1_ok and r2_ok:
                        if c == qr.y1:
                            poss[other(need_dir)] = false
                    else:
                        let lo_x = min(qr.x1, qr.x2)
                        let hi_x = max(qr.x1, qr.x2)
                        if not any((r in mask) != go_left for r in lo_x.r..hi_x.l):
                            continue 'state
                        poss[other(need_dir)] = false

                var cur: ?Int = none
                for d in VDir if poss[d]:
                    let next_other = last[other(d)]
                    let step_cost = if d == vert[c]: 0 else: costv[c]
                    let cand = step_cost + dp[c.r, next_other, d]
                    if let v = cand:
                        if cur == none or v < cur!:
                            cur = v
                            nxt[c, prev_other, ltype] = (d, next_other, d)
                dp[c.l, prev_other, ltype] = cur

    let base_cost = dp[Col.first.l, none, N]
    let h_cost = sum(costh[r] for r in Row if (r in mask) != hori[r])
    if let total = base_cost + h_cost:
        if best_cost == none or total < best_cost!:
            best_cost = total
            best_h = Str([if r in mask: 'E' else: 'W' for r in Row])
            var out_v = ""
            var cur_other: ?Col = none
            var cur_dir = N
            for c in Col:
                let (chosen, no, nd) = nxt[c, cur_other, cur_dir]
                out_v.push(if chosen == S: 'S' else: 'N')
                (cur_other, cur_dir) = (no, nd)
            best_v = out_v

if let ans = best_cost:
    print("possible")
    print(ans)
    print(best_h)
    print(best_v)
else:
    print("impossible")
