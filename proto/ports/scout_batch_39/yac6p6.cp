// YAC 6 P6 — original: corpus/submissions/5150025.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

const MAXK = 5

let (n, m, q): (1..=200, 1..=200, 1..=200_000) = read()
domain Row = 1..=n
domain Col = 1..=m
let a: [Row, Col]Int = read()

type Query = (x1: Row, y1: Col, x2: Row, y2: Col, kv: 0..=MAXK, id: Int)
var qans: [0..q]?Int = none
var buckets: [Row, Col]List<Query> = [Row, Col] (_, _) => []

// Small sub-rectangle DP directly from (x1, y1) to (x2, y2)
fn small_dp(x1: Row, y1: Col, x2: Row, y2: Col, kv: Int) -> Int:
    // FRICTION 39.4: mixing nominal Row/Col with structural 0..=MAXK for skip-count arithmetic
    var dp: [Row, Col, 0..=MAXK]?Int = none
    for r in (x1..=x2).rev:
        for c in (y1..=y2).rev:
            if r == x2 and c == y2:
                for k in 0..kv: dp[r, c, k] = 0
                dp[r, c, kv] = a[r, c]
                continue
            for k in 0..=kv:
                // FRICTION 39.4: ?Int + Int requires an `if let` guard on every DP transition
                if let nr = r + 1 if nr <= x2:
                    if let nxt = dp[nr, c, k]: dp[r, c, k] min= nxt + a[r, c]
                    if k < kv: dp[r, c, k] min= dp[nr, c, k + 1]
                if let nc = c + 1 if nc <= y2:
                    if let nxt = dp[r, nc, k]: dp[r, c, k] min= nxt + a[r, c]
                    if k < kv: dp[r, c, k] min= dp[r, nc, k + 1]
    dp[x1, y1, 0]!

for id in 0..q:
    let (x1, y1, x2, y2, kv): (Row, Col, Row, Col, 0..=MAXK) = read()
    if x2 - x1 <= 3 and y2 - y1 <= 3:
        qans[id] = small_dp(x1, y1, x2, y2, kv)
        continue
    let rem = (x2.to_idx + y2.to_idx) % 3
    for dx in 0..=rem:
        let dy = rem - dx
        if let cx = x2 - dx and let cy = y2 - dy:
            if cx >= x1 and cy >= y1:
                buckets[cx, cy].push((x1, y1, x2, y2, kv, id))

var dp: [Row, Col, 0..=MAXK]?Int = none
var dp2: [0..3, 0..3, 0..=MAXK]?Int = none

for cx in Row:
    for cy in Col if not buckets[cx, cy].is_empty:
        // Forward 3x3 suffix DP excluding (cx, cy)
        dp2.fill(none)
        for k in 0..=MAXK: dp2[0, 0, k] = 0
        for dr in 0..3:
            for dc in 0..3 if dr > 0 or dc > 0:
                if let r = cx + dr and let c = cy + dc:
                    for k in 0..=MAXK:
                        if dr > 0:
                            if let prev = dp2[dr - 1, dc, k]: dp2[dr, dc, k] min= prev + a[r, c]
                            if k > 0: dp2[dr, dc, k] min= dp2[dr - 1, dc, k - 1]
                        if dc > 0:
                            if let prev = dp2[dr, dc - 1, k]: dp2[dr, dc, k] min= prev + a[r, c]
                            if k > 0: dp2[dr, dc, k] min= dp2[dr, dc - 1, k - 1]

        // Backward prefix DP up to (cx, cy) inclusive
        for r in (..=cx).rev:
            for c in (..=cy).rev:
                for k in 0..=MAXK: dp[r, c, k] = none
                if r == cx and c == cy:
                    dp[r, c, 0] = a[r, c]
                    for k in 1..=MAXK: dp[r, c, k] = 0
                    continue
                for k in 0..=MAXK:
                    if let nr = r + 1 if nr <= cx:
                        if let nxt = dp[nr, c, k]: dp[r, c, k] min= nxt + a[r, c]
                        if k > 0: dp[r, c, k] min= dp[nr, c, k - 1]
                    if let nc = c + 1 if nc <= cy:
                        if let nxt = dp[r, nc, k]: dp[r, c, k] min= nxt + a[r, c]
                        if k > 0: dp[r, c, k] min= dp[r, nc, k - 1]

        for qry in buckets[cx, cy]:
            let (dr, dc) = (qry.x2 - cx, qry.y2 - cy)
            for k in 0..=qry.kv:
                if let p1 = dp[qry.x1, qry.y1, k] and let p2 = dp2[dr, dc, qry.kv - k]:
                    qans[qry.id] min= p1 + p2

for id in 0..q:
    print(qans[id]!)
