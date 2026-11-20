// IOI '11 P5 — Dancing Elephants (runnable proto/cpc port of ioi11p5_draft.cp)
// Prototype deltas vs v2.1 draft: no struct field defaults, no `for mut`,
// no `List.filled`/`List.insert`/`List.remove`, standalone stdio driver for differential testing.

let n: 1..=150_000 = read()
let l: 0..=1_000_000_000 = read()
let q: 1..=150_000 = read()

domain E = 0..n
var pos: [E]Int = read()

var B = 1
while B * B < n:
    B += 1
if B < 2:
    B = 2

var bxs: List<List<Int>> = []
var bend: List<List<Int>> = []
var bcnt: List<List<Int>> = []

fn rebuild_bucket(bi: Int):
    let k = bxs[bi].len
    bend[bi].clear()
    bcnt[bi].clear()
    for _ in 0..k:
        bend[bi].push(0)
        bcnt[bi].push(0)
    var j = k - 1
    for i in (0..k).rev:
        while j > i and bxs[bi][j] > bxs[bi][i] + l:
            j -= 1
        let nxt = j + 1
        if nxt >= k:
            bend[bi][i] = bxs[bi][i] + l
            bcnt[bi][i] = 1
        else:
            bend[bi][i] = bend[bi][nxt]
            bcnt[bi][i] = bcnt[bi][nxt] + 1

fn rebuild_all():
    var all: List<Int> = []
    for e in E:
        all.push(pos[e])
    all.sort()
    bxs.clear()
    bend.clear()
    bcnt.clear()
    var i = 0
    while i < n:
        var cur: List<Int> = []
        var j = i
        while j < n and j < i + B:
            cur.push(all[j])
            j += 1
        bxs.push(cur)
        var e1: List<Int> = []
        var c1: List<Int> = []
        bend.push(e1)
        bcnt.push(c1)
        rebuild_bucket(bxs.len - 1)
        i = j

fn bucket_remove(bi: Int, x: Int):
    let k = bxs[bi].len
    var idx = 0
    while idx < k and bxs[bi][idx] != x:
        idx += 1
    var i = idx
    while i + 1 < k:
        bxs[bi][i] = bxs[bi][i + 1]
        i += 1
    bxs[bi].pop()
    rebuild_bucket(bi)

fn bucket_insert(bi: Int, x: Int):
    bxs[bi].push(x)
    var i = bxs[bi].len - 1
    while i > 0 and bxs[bi][i - 1] > bxs[bi][i]:
        let t = bxs[bi][i]
        bxs[bi][i] = bxs[bi][i - 1]
        bxs[bi][i - 1] = t
        i -= 1
    rebuild_bucket(bi)

fn first_gt_idx(bi: Int, lim: Int) -> Int:
    var lo = 0
    var hi = bxs[bi].len
    while lo < hi:
        let mid = (lo + hi) / 2
        if bxs[bi][mid] > lim:
            hi = mid
        else:
            lo = mid + 1
    return lo

rebuild_all()
var ops = 0

for _ in 0..q:
    let idx_raw: Int = read()
    let y: Int = read()
    let e = E.from_idx(idx_raw)!
    let old_x = pos[e]
    pos[e] = y

    var rem_bi = bxs.len - 1
    for bi in 0..bxs.len:
        let k = bxs[bi].len
        if k > 0 and bxs[bi][0] <= old_x and old_x <= bxs[bi][k - 1]:
            rem_bi = bi
            break
    bucket_remove(rem_bi, old_x)

    var ins_bi = bxs.len - 1
    for bi in 0..bxs.len:
        let k = bxs[bi].len
        if k > 0 and y <= bxs[bi][k - 1]:
            ins_bi = bi
            break
    bucket_insert(ins_bi, y)

    ops += 1
    if ops >= B:
        ops = 0
        rebuild_all()

    var ans = 0
    var has_cover = false
    var cover = 0
    for bi in 0..bxs.len:
        let k = bxs[bi].len
        if k == 0:
            continue
        let start = if not has_cover: 0 else: first_gt_idx(bi, cover)
        if start < k:
            ans += bcnt[bi][start]
            cover = bend[bi][start]
            has_cover = true
    print(ans)
