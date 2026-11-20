// YAC 6 P6 (today/ runnable subset) — original: corpus/submissions/5150025.cpp

const MAXK = 5
const INF = 1_000_000_000_000_000_000

let n: 1..=200 = read()
let m: 1..=200 = read()
let q: 1..=200_000 = read()
var a: [0..n, 0..m]Int = 0
for i in 0..n:
    for j in 0..m:
        let v: Int = read()
        a[i, j] = v

var qans: [0..q]Int = INF
var b_x1: [0..n, 0..m]List<Int> = []
var b_y1: [0..n, 0..m]List<Int> = []
var b_x2: [0..n, 0..m]List<Int> = []
var b_y2: [0..n, 0..m]List<Int> = []
var b_kv: [0..n, 0..m]List<Int> = []
var b_id: [0..n, 0..m]List<Int> = []

fn small_dp(x1: Int, y1: Int, x2: Int, y2: Int, kv: Int) -> Int:
    let h = x2 - x1 + 1
    let w = y2 - y1 + 1
    var dp: [0..h, 0..w, 0..=MAXK]Int = INF
    for i in (0..h).rev:
        for j in (0..w).rev:
            if i == h - 1 and j == w - 1:
                for k in 0..kv:
                    dp[i, j, k] = 0
                dp[i, j, kv] = a[x1 + i, y1 + j]
                continue
            let cost = a[x1 + i, y1 + j]
            for k in 0..=kv:
                var best = INF
                if i + 1 < h:
                    best min= cost + dp[i + 1, j, k]
                    if k + 1 <= kv:
                        best min= dp[i + 1, j, k + 1]
                if j + 1 < w:
                    best min= cost + dp[i, j + 1, k]
                    if k + 1 <= kv:
                        best min= dp[i, j + 1, k + 1]
                dp[i, j, k] = best
    dp[0, 0, 0]

for id in 0..q:
    let x1_in: 1..=200 = read()
    let y1_in: 1..=200 = read()
    let x2_in: 1..=200 = read()
    let y2_in: 1..=200 = read()
    let kv: 0..=MAXK = read()
    let x1 = x1_in - 1
    let y1 = y1_in - 1
    let x2 = x2_in - 1
    let y2 = y2_in - 1
    if x2 - x1 <= 3 and y2 - y1 <= 3:
        qans[id] = small_dp(x1, y1, x2, y2, kv)
        continue
    let rem = (x2 + y2) % 3
    for dx in 0..=rem:
        let dy = rem - dx
        let cx = x2 - dx
        let cy = y2 - dy
        if cx >= x1 and cy >= y1:
            b_x1[cx, cy].push(x1)
            b_y1[cx, cy].push(y1)
            b_x2[cx, cy].push(x2)
            b_y2[cx, cy].push(y2)
            b_kv[cx, cy].push(kv)
            b_id[cx, cy].push(id)

var dp: [0..n, 0..m, 0..=MAXK]Int = INF
var dp2: [0..3, 0..3, 0..=MAXK]Int = INF

for cx in 0..n:
    for cy in 0..m:
        if b_id[cx, cy].len == 0:
            continue
        // 3x3 forward suffix DP excluding (cx, cy)
        for dr in 0..3:
            for dc in 0..3:
                let r = cx + dr
                let c = cy + dc
                if r >= n or c >= m:
                    continue
                if dr == 0 and dc == 0:
                    for k in 0..=MAXK:
                        dp2[0, 0, k] = 0
                    continue
                for k in 0..=MAXK:
                    var best = INF
                    if dr > 0:
                        best min= a[r, c] + dp2[dr - 1, dc, k]
                        if k > 0:
                            best min= dp2[dr - 1, dc, k - 1]
                    if dc > 0:
                        best min= a[r, c] + dp2[dr, dc - 1, k]
                        if k > 0:
                            best min= dp2[dr, dc - 1, k - 1]
                    dp2[dr, dc, k] = best

        // Backward prefix DP up to (cx, cy) inclusive
        for r in (0..=cx).rev:
            for c in (0..=cy).rev:
                if r == cx and c == cy:
                    dp[r, c, 0] = a[r, c]
                    for k in 1..=MAXK:
                        dp[r, c, k] = 0
                    continue
                let cost = a[r, c]
                for k in 0..=MAXK:
                    var best = INF
                    if r + 1 <= cx:
                        best min= cost + dp[r + 1, c, k]
                        if k > 0:
                            best min= dp[r + 1, c, k - 1]
                    if c + 1 <= cy:
                        best min= cost + dp[r, c + 1, k]
                        if k > 0:
                            best min= dp[r, c + 1, k - 1]
                    dp[r, c, k] = best

        let cnt = b_id[cx, cy].len
        for qi in 0..cnt:
            let qx1 = b_x1[cx, cy][qi]
            let qy1 = b_y1[cx, cy][qi]
            let qx2 = b_x2[cx, cy][qi]
            let qy2 = b_y2[cx, cy][qi]
            let qkv = b_kv[cx, cy][qi]
            let qid = b_id[cx, cy][qi]
            let dr = qx2 - cx
            let dc = qy2 - cy
            for k in 0..=qkv:
                qans[qid] min= dp[qx1, qy1, k] + dp2[dr, dc, qkv - k]

for id in 0..q:
    print(qans[id])
