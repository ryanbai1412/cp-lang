// Runnable today/ port of NOI '19 P3 (Sequence) for real Rust `cpl build`.
// Stdin: t testcases, each with n, k, l, then a[0..n], b[0..n]. Prints max sum per case.
// original: corpus/submissions/5669058.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

fn next_perm(mut a: List<Int>) -> Bool:
    let n = a.len
    var i = n - 2
    while i >= 0 and a[i] >= a[i + 1]: i -= 1
    if i < 0: return false
    var j = n - 1
    while a[j] <= a[i]: j -= 1
    a.swap(i, j)
    var l = i + 1
    var r = n - 1
    while l < r:
        a.swap(l, r)
        l += 1
        r -= 1
    return true

fn solve_one():
    let n: Int = read()
    let k: Int = read()
    let l: Int = read()
    var a: List<Int> = []
    var b: List<Int> = []
    for _ in 0..n: a.push(ri())
    for _ in 0..n: b.push(ri())

    // 13 max-heaps: 0..12 for (s, t) transitions (s in 0..4, t in 0..3), 12 for worst in bucket 2 (negated).
    var h_val: List<List<Int>> = []
    var h_idx: List<List<Int>> = []
    for _ in 0..13:
        h_val.push([])
        h_idx.push([])

    var st = [0..n] _ => 3
    var sz = [0..4] _ => 0
    var cap = [0, 0, k]
    var flow = 0

    fn val(s: Int, i: Int) -> Int:
        if s == 0: return a[i]
        if s == 1: return b[i]
        if s == 2: return a[i] + b[i]
        return 0

    fn h_push(id: Int, v: Int, idx: Int):
        h_val[id].push(v)
        h_idx[id].push(idx)
        var i = h_val[id].len - 1
        while i > 0:
            let p = (i - 1) / 2
            if h_val[id][p] >= h_val[id][i]: break
            h_val[id].swap(p, i)
            h_idx[id].swap(p, i)
            i = p

    fn h_pop(id: Int):
        let lv = h_val[id].pop()!
        let li = h_idx[id].pop()!
        if h_val[id].len > 0:
            h_val[id][0] = lv
            h_idx[id][0] = li
            var i = 0
            while true:
                var big = i
                let lc = i * 2 + 1
                let rc = i * 2 + 2
                if lc < h_val[id].len and h_val[id][lc] > h_val[id][big]: big = lc
                if rc < h_val[id].len and h_val[id][rc] > h_val[id][big]: big = rc
                if big == i: break
                swap(mut h_val[id][big], mut h_val[id][i])
                swap(mut h_idx[id][big], mut h_idx[id][i])
                i = big

    fn top_pair(s: Int, t: Int) -> (Int, Int):
        let id = s * 3 + t
        while h_val[id].len > 0 and st[h_idx[id][0]] != s:
            h_pop(id)
        if h_val[id].len == 0: return (-1_000_000_000_000_000, -1)
        return (h_val[id][0], h_idx[id][0])

    fn worst_two() -> Int:
        while h_val[12].len > 0 and st[h_idx[12][0]] != 2:
            h_pop(12)
        return h_idx[12][0]

    fn push_all(i: Int, s: Int):
        if s == 2: h_push(12, -(a[i] + b[i]), i)
        for t in 0..3:
            if t != s: h_push(s * 3 + t, val(t, i) - val(s, i), i)

    for i in 0..n:
        sz[3] += 1
        push_all(i, 3)

    fn move_item(i: Int, to: Int):
        let fr = st[i]
        if fr < 3: cap[fr] += 1
        if to < 3: cap[to] -= 1
        flow += val(to, i) - val(fr, i)
        sz[fr] -= 1
        sz[to] += 1
        st[i] = to
        push_all(i, to)

    var init_ord: List<(Int, Int)> = []
    for i in 0..n: init_ord.push((-(a[i] + b[i]), i))
    init_ord.sort()
    for idx in 0..k: move_item(init_ord[idx].1, 2)

    fn augment():
        var best = -1_000_000_000_000_000
        var best_path: List<Int> = []
        var perm = [0, 1, 2]
        while true:
            var prv = 3
            var gain = 0
            var path: List<Int> = []
            var ok = true
            for nxt in perm:
                let p = top_pair(prv, nxt)
                if p.1 == -1:
                    ok = false
                    break
                gain += p.0
                path.push(nxt)
                if cap[nxt] > 0: break
                if sz[nxt] == 0:
                    ok = false
                    break
                prv = nxt
            if ok and cap[path.last!] > 0 and gain > best:
                best = gain
                best_path = path
            if not next_perm(mut perm): break
        var prv = 3
        for nxt in best_path:
            let p = top_pair(prv, nxt)
            move_item(p.1, nxt)
            prv = nxt

    var ans = flow
    while sz[2] > l and sz[3] > 0:
        move_item(worst_two(), 3)
        cap[2] -= 1
        cap[0] += 1
        cap[1] += 1
        augment()
        augment()
        ans max= flow

    print(ans)

fn main_solve():
    let t: Int = read()
    for _ in 0..t: solve_one()

main_solve()
