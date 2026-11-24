// Source: EGOI 2022 P4 (Tourists), https://oj.uz/problem/view/EGOI22_tourists
// limits: 4000 ms, 256 MB
// Problem: a tree of n <= 2e5 cities; m <= 2e5 tourists start in cities a_i with opinion 0.
//   q <= 2e5 queries: "t f g c" moves tourists f..g to city c, each losing its path length;
//   "e c d" adds d <= 1e9 to everyone now in city c; "q v" prints tourist v's opinion.
// Solution: tourists with consecutive numbers in one city form intervals (an assign-range map
//   whose starts sit in a max segment tree); a 't' visits the intervals it overlaps and adds
//   O(1) new ones, so the work is amortized. Each city keeps a treap of its tourists keyed by
//   number with lazy add: a moved range is split out of the old city's treap, gets
//   -dist(old, c) and is merged into c's treap; an event adds at c's root, and a query walks
//   the treap of the city covering v, summing lazies. O((m + q) log m) amortized.
// Uses Rooted for O(1) tree dist, a max-SegTree ODT for O(log m) interval lookup/split,
// and flat parallel arrays for the per-city lazy-add treap arena.
// original: corpus/submissions/5199667.cpp

fn main_solve():
    let n: Int = read()
    let m: Int = read()
    let q: Int = read()
    domain City = 0..n
    domain Pos = 0..m

    var init_city: [Pos]City = City(0)
    for i in 0..m:
        let c: Int = read()
        init_city[Pos(i)] = City(c - 1)

    var adj: [City]List<City> = []
    for _ in 0..n - 1:
        let u: Int = read()
        let v: Int = read()
        adj[City(u - 1)].push(City(v - 1))
        adj[City(v - 1)].push(City(u - 1))
    let tree = Rooted(adj, City(0))

    // Deterministic priority generator (LCG within positive signed 64-bit)
    var rng = 141214121
    fn next_pri() -> Int:
        rng = (rng * 1103515245 + 12345) % 2147483647
        return rng

    // Flat arena for per-city lazy-add treap (parallel arrays avoid Opt<Int> in struct)
    var t_pos: List<Int> = []
    var t_pri: List<Int> = []
    var t_val: List<Int> = []
    var t_lazy: List<Int> = []
    var t_lc: List<Int> = []
    var t_rc: List<Int> = []
    var root: [City]Int = -1

    fn new_node(p: Int) -> Int:
        let id = t_pos.len
        t_pos.push(p)
        t_pri.push(next_pri())
        t_val.push(0)
        t_lazy.push(0)
        t_lc.push(-1)
        t_rc.push(-1)
        return id

    fn push(u: Int):
        let d = t_lazy[u]
        if d == 0: return
        t_val[u] += d
        let l = t_lc[u]
        let r = t_rc[u]
        if l != -1: t_lazy[l] += d
        if r != -1: t_lazy[r] += d
        t_lazy[u] = 0

    fn split(u: Int, key: Int) -> (Int, Int):
        if u == -1: return (-1, -1)
        push(u)
        if key <= t_pos[u]:
            let lc = t_lc[u]
            let res = split(lc, key)
            t_lc[u] = res.1
            return (res.0, u)
        else:
            let rc = t_rc[u]
            let res = split(rc, key)
            t_rc[u] = res.0
            return (u, res.1)

    fn merge(a: Int, b: Int) -> Int:
        if a == -1: return b
        if b == -1: return a
        if t_pri[a] < t_pri[b]:
            push(a)
            let rc = t_rc[a]
            let nr = merge(rc, b)
            t_rc[a] = nr
            return a
        else:
            push(b)
            let lc = t_lc[b]
            let nl = merge(a, lc)
            t_lc[b] = nl
            return b

    fn move_add(l: Int, r: Int, old_c: City, new_c: City, delta: Int):
        let r_old = root[old_c]
        let s1 = split(r_old, l)
        let s2 = split(s1.1, r + 1)
        root[old_c] = merge(s1.0, s2.1)
        let mid = s2.0
        if mid != -1: t_lazy[mid] += delta
        let r_new = root[new_c]
        let t1 = split(r_new, l)
        root[new_c] = merge(merge(t1.0, mid), t1.1)

    fn query_energy(p: Int, c: City) -> Int:
        var cur = root[c]
        var acc = 0
        while cur != -1:
            acc += t_lazy[cur]
            if p == t_pos[cur]: return acc + t_val[cur]
            if p < t_pos[cur]: cur = t_lc[cur]
            else: cur = t_rc[cur]
        return 0

    // ODT backed by max-SegTree of active interval starts (O(log m) last_le)
    var odt_st = SegTree<Pos, Int>(max, fill: -1)
    var seg_r: [Pos]Int = -1
    var seg_c: [Pos]City = City(0)

    for i in 0..m:
        let p = Pos(i)
        let c = init_city[p]
        seg_r[p] = i
        seg_c[p] = c
        odt_st.set(p, i)
        let id = new_node(i)
        root[c] = merge(root[c], id)

    fn set_seg(l: Int, r: Int, c: City):
        let p = Pos(l)
        seg_r[p] = r
        seg_c[p] = c
        odt_st.set(p, l)

    fn del_seg(l: Int):
        odt_st.set(Pos(l), -1)

    fn assign_range(l: Int, r: Int, x: City):
        var cur = odt_st.fold(Pos(0)..=Pos(r))
        while cur != -1:
            let cl = cur
            let cr = seg_r[Pos(cl)]
            let px = seg_c[Pos(cl)]
            if cr < l: break
            if cl > 0: cur = odt_st.fold(Pos(0)..=Pos(cl - 1))
            else: cur = -1
            let ov_l = max(l, cl)
            let ov_r = min(r, cr)
            move_add(ov_l, ov_r, px, x, -tree.dist(px, x))
            if cl < l:
                seg_r[Pos(cl)] = l - 1
                if r < cr: set_seg(r + 1, cr, px)
            elif r < cr:
                del_seg(cl)
                set_seg(r + 1, cr, px)
            else:
                del_seg(cl)
        set_seg(l, r, x)

    for _ in 0..q:
        let op: Char = read()
        if op == 't':
            let l: Int = read()
            let r: Int = read()
            let x: Int = read()
            assign_range(l - 1, r - 1, City(x - 1))
        elif op == 'e':
            let c: Int = read()
            let v: Int = read()
            let u = root[City(c - 1)]
            if u != -1: t_lazy[u] += v
        else:
            let p: Int = read()
            let start = odt_st.fold(Pos(0)..=Pos(p - 1))
            print(query_energy(p - 1, seg_c[Pos(start)]))

main_solve()
