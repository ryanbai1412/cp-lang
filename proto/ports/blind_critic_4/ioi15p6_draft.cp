// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi15p6 (Towns) — Messy Ad-Hoc / Interactive / Non-Typeable Stress #2
// Original: corpus/submissions/5723344.cpp
// Exercises: lexically scoped @memo (auto-reset per call), max_by, Map<Int, Int>, Tropical Optionals, @grader / @interactor

@grader
fn get_distance(i: Int, j: Int) -> Int

@grader
fn hub_distance(n: Int, sub: Int) -> Int:
    domain V = 0..n
    let v0 = V.first

    // Lexically scoped @memo inside fn: fresh flat cache `[V, V]?Int` every call!
    // BLIND-GUESS(H18): does @memo work inside a regular fn (not just top-level/loop) AND allow calling extern @grader fn?
    @memo
    fn cached_dist(a: V, b: V) -> Int:
        get_distance(a.to_idx, b.to_idx)

    fn dist(u: V, v: V) -> Int:
        if u == v: 0
        elif u < v: cached_dist(u, v)
        else: cached_dist(v, u)

    // BLIND-GUESS(H19): can domain `V` be passed directly as an iterable to `max_by(V, ...)`?
    let u1 = max_by(V, i => dist(v0, i))!
    let u2 = max_by(V, i => dist(u1, i))!
    let diam = dist(u1, u2)

    var hori: [V]Int = 0
    var down: [V]Int = 0

    // Project vertex 0 onto diameter (u1, u2)
    let d1_0 = dist(u1, v0)
    let d2_0 = dist(u2, v0)
    down[v0] = (d1_0 + d2_0 - diam) / 2
    hori[v0] = d1_0 - down[v0]

    // Project every other vertex onto diameter using ONLY queries to v0 and u1
    for i in (v0 + 1)!..=V.last:
        let d1 = dist(u1, i)
        let d0 = dist(v0, i)
        let common = (d1 + d0 - d1_0) / 2
        let l0 = d0 - common
        let l1 = d1 - common
        if l0 <= down[v0]:
            hori[i] = hori[v0]
            down[i] = common + l1 - hori[v0]
        elif l1 < hori[v0]:
            hori[i] = l1
            down[i] = common
        else:
            hori[i] = d1
            down[i] = 0

    let best_r = min(max(hori[i], diam - hori[i]) for i in V)!

    // Group leaves by projection position along the diameter
    var groups = Map<Int, Int>()
    for i in V:
        groups[hori[i]] = (groups.get(hori[i]) ?? 0) + 1
        // HESITATION: §9 says `m[k] f= x` — does `groups[hori[i]] += 1` default-initialize missing keys to 0, or trap?

    var cands: List<Int> = []
    var cl = 0
    for (pos, cm) in groups.items:
        let cr = n - cl - cm
        if max(pos, diam - pos) == best_r:
            // BLIND-GUESS(H21): is max(a, b, c) variadic or 2-arg only? Using nested max to be safe!
            if max(cl, max(cm, cr)) <= n / 2:
                return best_r
            if max(cl, cr) <= n / 2:
                cands.push(pos)
        cl += cm

    if cands.is_empty:
        return -best_r

    let hub_pos = cands[0]
    let nodes = [i for i in V if hori[i] == hub_pos]

    fn same_branch(u: V, v: V) -> Bool:
        dist(u, v) != down[u] + down[v]

    // Boyer-Moore majority voting with segment pairing (<= 3/2 * |nodes| queries)
    var cur_same: List<V> = []
    var cur_diff: List<V> = []
    var segs: List<(same: List<V>, diff: List<V>)> = []

    for a in nodes:
        if cur_same.is_empty:
            cur_same.push(a)
        elif same_branch(cur_same[0], a):
            cur_same.push(a)
        else:
            cur_diff.push(a)
        if cur_same.len == cur_diff.len:
            segs.push((same: cur_same.take(), diff: cur_diff.take()))

    if cur_same.is_empty:
        return best_r

    let maj = cur_same[0]
    var maj_cnt = cur_same.len
    for seg in segs:
        if same_branch(seg.same[0], maj):
            maj_cnt += seg.same.len
        for a in seg.diff:
            if same_branch(a, maj):
                maj_cnt += 1

    if maj_cnt <= n / 2: best_r else: -best_r
