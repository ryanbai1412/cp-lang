// Runnable port (proto/cp.py lib-run) of statement-first dmpg15g6.cp
// original: corpus/submissions/4253124.cpp
// Deterministic Rollback Mo's algorithm (manual undo log simulating `Journaled` + `undo`).
let (n, m): (Int, Int) = read()
domain I = 1..=n
domain Flav = 1..=m
let a: [I]Flav = read()
let q: Int = read()
domain Q = 0..q
let qs: [Q](l: I, r: I) = read()

var blk = 1
while (blk + 1) * (blk + 1) <= n:
    blk += 1
let num_blk = (n - 1) / blk + 1

var ans: [Q]Bool = [Q] _ => false
var freq: [Flav]Int = [Flav] _ => 0
var log_f: List<Int> = []
var log_v: List<Int> = []

var f1 = -1
var c1 = 0
var f2 = -1
var c2 = 0

fn check_ok(len: Int) -> Bool:
    3 * c1 >= 2 * len or (3 * c1 >= len and 3 * c2 >= len)

fn add_pos(idx: Int):
    let fl = a[I.nth(idx)]
    let fid = fl.to_idx
    log_f.push(fid)
    log_v.push(freq[fl])
    let c = freq[fl] + 1
    freq[fl] = c
    if f1 == fid:
        c1 = c
    elif f2 == fid:
        c2 = c
        if c2 > c1:
            swap(mut f1, mut f2)
            swap(mut c1, mut c2)
    elif c > c1:
        f2 = f1
        c2 = c1
        f1 = fid
        c1 = c
    elif c > c2:
        f2 = fid
        c2 = c

fn rollback(mark: Int, sf1: Int, sc1: Int, sf2: Int, sc2: Int):
    while log_f.len > mark:
        let fid = log_f.last!
        let old = log_v.last!
        log_f.pop()
        log_v.pop()
        freq[Flav.nth(fid)] = old
    f1 = sf1
    c1 = sc1
    f2 = sf2
    c2 = sc2

var by_blk: List<List<(r: Int, qi: Int)>> = []
for _ in 0..num_blk:
    by_blk.push([])

for qi in Q:
    let l = qs[qi].l.to_idx
    let r = qs[qi].r.to_idx
    if r - l <= blk:
        let mk = log_f.len
        for p in l..=r: add_pos(p)
        ans[qi] = check_ok(r - l + 1)
        rollback(mk, -1, 0, -1, 0)
    else:
        by_blk[l / blk].push((r: r, qi: qi.to_idx))

for b in 0..num_blk:
    if by_blk[b].len == 0: continue
    let bucket = sorted(by_blk[b])
    let border = min(n, (b + 1) * blk)
    let outer_mk = log_f.len
    var cur_r = border - 1
    for item in bucket:
        let qi = Q(item.qi)
        let l = qs[qi].l.to_idx
        let r = qs[qi].r.to_idx
        while cur_r < r:
            cur_r += 1
            add_pos(cur_r)
        let inner_mk = log_f.len
        let sf1 = f1
        let sc1 = c1
        let sf2 = f2
        let sc2 = c2
        for p in l..border:
            add_pos(p)
        ans[qi] = check_ok(r - l + 1)
        rollback(inner_mk, sf1, sc1, sf2, sc2)
    rollback(outer_mk, -1, 0, -1, 0)

for qi in Q:
    if ans[qi]: print("YES")
    else: print("NO")
