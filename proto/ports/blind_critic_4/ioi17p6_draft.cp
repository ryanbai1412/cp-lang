// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi17p6 (Books) — Messy Ad-Hoc #1
// Original: corpus/submissions/5659304.cpp
// Exercises: Perm<V> (.cycles()), Marks<V>, Tropical Optionals (?V, min=), messy bidirectional hull expansion

@grader
fn minimum_walk(p_raw: List<Int>, s_raw: Int) -> Int:
    let n = p_raw.len
    domain V = 0..n
    let s = V.from_idx(s_raw)!
    let p: [V]V = [V] v => V.from_idx(p_raw[v.to_idx])!

    // Base mandatory carry distance: every book i must move |p[i] - i|
    var base = 0
    var min_need = s
    var max_need = s
    for v in V:
        base += abs(p[v] - v)
        if p[v] != v:
            min_need min= v
            max_need max= v

    if base == 0:
        return 0

    // BLIND-GUESS(H14): how is Perm<V> constructed from `[V]V`? Does `Perm<V>(p).cycles()` return `List<List<V>>`?
    var cyc_l: [V]V = [V] v => v
    var cyc_r: [V]V = [V] v => v
    for cyc in Perm<V>(p).cycles():
        let lo = min(cyc)!
        let hi = max(cyc)!
        for v in cyc:
            cyc_l[v] = lo
            cyc_r[v] = hi

    // Expand closed interval [l, r] to its cycle-connected hull using Marks<V>
    var expanded = Marks<V>()
    fn expand(mut l: V, mut r: V):
        var pl = l
        var pr = r
        // Queue-free outward scan from initial [l, r]
        var cur_l = l
        var cur_r = r
        // Mark and pull all cycles intersecting [cur_l, cur_r]
        var q: Deque<V> = []
        for v in l..=r:
            if expanded.add(v):
                q.push(v)
        while let u = q.pop_front():
            let nl = cyc_l[u]
            let nr = cyc_r[u]
            if nl < cur_l:
                for v in nl..cur_l:
                    if expanded.add(v):
                        q.push(v)
                cur_l = nl
            if nr > cur_r:
                for v in (cur_r + 1)!..=nr:
                    if expanded.add(v):
                        q.push(v)
                cur_r = nr
        l = cur_l
        r = cur_r

    // Start from the closed cycle hull containing s
    var l = cyc_l[s]
    var r = cyc_r[s]
    expand(mut l, mut r)

    var bridge = 0

    while l > min_need or r < max_need:
        // Probe left until crossing > r or reaching min_need
        var ll = l
        var lr = r
        var cost_l = 0
        while ll > min_need and lr == r:
            ll = (ll - 1)!
            cost_l += 1
            // Expand locally without mutating outer [l, r]; use a fresh probe or track bounds
            // HESITATION: expanding both left and right probes shares `expanded` Marks<V>!
            // Wait: because probes touch disjoint regions outside [l, r] until hitting an enclosing cycle,
            // we actually must NOT mark the enclosing cycle's other side before probing the right!
            // So use two-pointer hull expansion purely on (ll, lr) bounds:
            var nl = cyc_l[ll]
            var nr = cyc_r[ll]
            var scan = ll
            while scan > nl or nr > lr:
                if scan > nl:
                    scan = (scan - 1)!
                    nl min= cyc_l[scan]
                    nr max= cyc_r[scan]
                else:
                    lr = nr
                    break
            ll = nl
            lr max= nr

        // Probe right until crossing < l or reaching max_need
        var rl = l
        var rr = r
        var cost_r = 0
        while rr < max_need and rl == l:
            rr = (rr + 1)!
            cost_r += 1
            var nl = cyc_l[rr]
            var nr = cyc_r[rr]
            var scan = rr
            while scan < nr or nl < rl:
                if scan < nr:
                    scan = (scan + 1)!
                    nl min= cyc_l[scan]
                    nr max= cyc_r[scan]
                else:
                    rl = nl
                    break
            rr = nr
            rl min= nl

        if lr > r:
            // An enclosing cycle wraps [l, r]; left and right probes meet the same enclosing hull!
            bridge += min(cost_l, cost_r)
            l = min(ll, rl)
            r = max(lr, rr)
            expand(mut l, mut r)
        else:
            // Left and right sides are completely disjoint: must bridge BOTH
            bridge += cost_l + cost_r
            break

    return base + 2 * bridge
