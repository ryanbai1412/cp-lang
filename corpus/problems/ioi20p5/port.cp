// Runnable today/ port of IOI '20 P5 (Mushrooms) for real Rust `cpl build`.
// Stdin: n, then hidden binary mushroom types s[0..n] (0 = A, 1 = B; s[0] == 0).
// Simulates use_machine, runs the decision-tree + interleaving solver, prints count of A.
// original: corpus/submissions/4736004.cpp

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

fn eval_mask(mask: Int, perm: List<Int>) -> Int:
    var ans = 0
    var prev = -1
    for v in perm:
        let cur = if v < 2: v else: (mask >> (v - 2)) & 1
        if prev != -1 and cur != prev: ans += 1
        prev = cur
    return ans

fn make_base_perm(k: Int, x: Int, y: Int) -> List<Int>:
    var p: List<Int> = [i + 2 for i in 0..k]
    for _ in 0..x: p.push(0)
    for _ in 0..y: p.push(1)
    return p

fn compute_dt(k: Int, x: Int, y: Int) -> (List<Int>, List<List<Int>>, List<List<Int>>):
    let tot = k + x + y
    var p = make_base_perm(k, x, y)
    while true:
        var buckets: List<List<Int>> = [[] for _ in 0..tot]
        for mask in 0..(1 << k):
            buckets[eval_mask(mask, p)].push(mask)
        var leaf_q: List<List<Int>> = []
        var leaf_res: List<List<Int>> = []
        var ok_root = true
        for grp in buckets:
            var p2 = make_base_perm(k, x, y)
            var found_leaf = false
            while true:
                var tbl = [0..tot] _ => -1
                var ok_leaf = true
                for m in grp:
                    let r = eval_mask(m, p2)
                    if tbl[r] != -1:
                        ok_leaf = false
                        break
                    tbl[r] = m
                if ok_leaf:
                    var qcopy: List<Int> = [z for z in p2]
                    leaf_q.push(qcopy)
                    leaf_res.push(tbl)
                    found_leaf = true
                    break
                if not next_perm(mut p2): break
            if not found_leaf:
                ok_root = false
                break
        if ok_root:
            var rcopy: List<Int> = [z for z in p]
            return (rcopy, leaf_q, leaf_res)
        if not next_perm(mut p): break
    return ([], [], [])

fn main_solve():
    let n: Int = read()
    var s: List<Int> = [ri() for _ in 0..n]

    fn use_machine(seq: List<Int>) -> Int:
        var ans = 0
        for i in 1..seq.len:
            if s[seq[i]] != s[seq[i - 1]]: ans += 1
        return ans

    var known0: List<Int> = [0]
    var known1: List<Int> = []
    var pos = 1
    let bs = 100

    var cache_key: List<(Int, Int, Int)> = []
    var cache_r: List<List<Int>> = []
    var cache_lq: List<List<List<Int>>> = []
    var cache_lr: List<List<List<Int>>> = []

    fn get_dt(k: Int, x: Int, y: Int) -> Int:
        for i in 0..cache_key.len:
            let ck = cache_key[i]
            if ck.0 == k and ck.1 == x and ck.2 == y: return i
        let dt = compute_dt(k, x, y)
        let id = cache_key.len
        cache_key.push((k, x, y))
        cache_r.push(dt.0)
        cache_lq.push(dt.1)
        cache_lr.push(dt.2)
        return id

    fn ask_pattern(pat: List<Int>, vars: List<Int>) -> Int:
        var seq: List<Int> = []
        var p0 = 0
        var p1 = 0
        for v in pat:
            if v == 0:
                seq.push(known0[p0])
                p0 += 1
            elif v == 1:
                seq.push(known1[p1])
                p1 += 1
            else:
                seq.push(vars[v - 2])
        return use_machine(seq)

    fn classify(want: Int, x: Int, y: Int):
        let k = min(want, n - pos)
        if k <= 0: return
        let id = get_dt(k, x, y)
        var vars: List<Int> = [pos + i for i in 0..k]
        pos += k
        let r1 = ask_pattern(cache_r[id], vars)
        let lq = cache_lq[id][r1]
        let lres = cache_lr[id][r1]
        var poss: List<Int> = []
        for m in lres:
            if m != -1: poss.push(m)
        let mask = if poss.len == 1: poss[0] else: lres[ask_pattern(lq, vars)]
        for i in 0..k:
            if ((mask >> i) & 1) == 0: known0.push(vars[i])
            else: known1.push(vars[i])

    while pos < n and pos < 2 * bs - 1:
        let a = known0.len
        let b = known1.len
        if a == 1 and b == 0: classify(1, 1, 0)
        elif a + b == 2: classify(2, a, b)
        elif a > 0 and b > 0:
            let ax = min(a, 3)
            classify(5, ax, 4 - ax)
        else:
            classify(4, 4, 0)

    var ans = known0.len
    while pos < n:
        let use_b = known1.len > known0.len
        let pool = if use_b: known1 else: known0
        let take = min(pool.len, n - pos)
        var seq: List<Int> = []
        for i in 0..take:
            seq.push(pool[i])
            seq.push(pos + i)
        let res = use_machine(seq)
        let flip = (res + 1) / 2
        ans += if use_b: flip else: take - flip
        if (res & 1) == 1:
            if use_b: known0.push(pos + take - 1)
            else: known1.push(pos + take - 1)
        else:
            if use_b: known1.push(pos + take - 1)
            else: known0.push(pos + take - 1)
        pos += take

    print(ans)

main_solve()
