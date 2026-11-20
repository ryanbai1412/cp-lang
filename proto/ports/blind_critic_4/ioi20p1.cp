// IOI '20 P1 — Plants (runnable proto/cpc port of ioi20p1_draft.cp, post-Drill-2 upgrade)
// Prototype deltas vs v2.1 draft: no generic structs, no `Action(...)` custom LazySegTree,
// uses built-in `AddMax<V>` (storing `-r[i]` so `r[i] == 0` iff value == 0) + `Set<Int>` + binary lifting.

let n: 2..=200_000 = read()
let k: 2..=200_000 = read()
let q: 1..=200_000 = read()

domain V = 0..n
var r_init: [V]Int = read()

// AddMax<V> stores `-r[i]` for active plants and `-INF` for extracted/placed plants.
let INF = 1_000_000_000
var st = AddMax<V>()
for i in 0..n:
    let vi = V.from_idx(i)
    st.add(vi, vi, -r_init[vi])

// Ordered sets `zeros` and `ready` via SegTree<V, Int> (proto/cpc Set<T> lacks first_ge/last_le).
var zero_cnt = 0
var zero_max = SegTree<V, Int>(-1, (a, b) => if a > b: a else: b)
var zero_min = SegTree<V, Int>(n, (a, b) => if a < b: a else: b)
var ready_min = SegTree<V, Int>(n, (a, b) => if a < b: a else: b)

fn prev_in_zeros(x: Int) -> Int:
    if x > 0:
        let p = zero_max.query(V.from_idx(0), V.from_idx(x - 1))
        if p >= 0:
            return p
    return zero_max.all

fn next_in_zeros(x: Int) -> Int:
    if x + 1 < n:
        let nx = zero_min.query(V.from_idx(x + 1), V.from_idx(n - 1))
        if nx < n:
            return nx
    return zero_min.all

fn cyc_dist(a: Int, b: Int) -> Int:
    var d = b - a
    if d < 0:
        d += n
    return d

fn check_ready(x: Int):
    let vx = V.from_idx(x)
    if zero_cnt == 1:
        ready_min.set(vx, x)
        return
    let prv = prev_in_zeros(x)
    if cyc_dist(prv, x) >= k:
        ready_min.set(vx, x)
    else:
        ready_min.set(vx, n)

fn add_zero(x: Int):
    let vx = V.from_idx(x)
    zero_cnt += 1
    zero_max.set(vx, x)
    zero_min.set(vx, x)
    check_ready(x)
    if zero_cnt > 1:
        let nxt = next_in_zeros(x)
        check_ready(nxt)

fn remove_zero(x: Int):
    let vx = V.from_idx(x)
    ready_min.set(vx, n)
    if zero_cnt == 1:
        zero_cnt -= 1
        zero_max.set(vx, -1)
        zero_min.set(vx, n)
        return
    let nxt = next_in_zeros(x)
    zero_cnt -= 1
    zero_max.set(vx, -1)
    zero_min.set(vx, n)
    check_ready(nxt)

// Find any active position with value == 0 in [l..=r] using AddMax<V>.
fn harvest_zeros(l: Int, r: Int):
    if l > r:
        return
    while st.max(V.from_idx(l), V.from_idx(r)) == 0:
        var lo = l
        var hi = r
        while lo < hi:
            let mid = (lo + hi) / 2
            if st.max(V.from_idx(lo), V.from_idx(mid)) == 0:
                hi = mid
            else:
                lo = mid + 1
        let vl = V.from_idx(lo)
        st.add(vl, vl, -INF)
        add_zero(lo)

fn add_cyc(l_raw: Int, r_raw: Int, delta: Int):
    var l = l_raw % n
    var r = r_raw % n
    if l < 0:
        l += n
    if r < 0:
        r += n
    if l <= r:
        st.add(V.from_idx(l), V.from_idx(r), delta)
        harvest_zeros(l, r)
    else:
        st.add(V.from_idx(l), V.from_idx(n - 1), delta)
        st.add(V.from_idx(0), V.from_idx(r), delta)
        harvest_zeros(l, n - 1)
        harvest_zeros(0, r)

harvest_zeros(0, n - 1)

// Extract plants in descending height order (rank n-1 down to 0).
var order: List<Int> = []
var height: [V]Int = 0
for step in 0..n:
    let x = ready_min.all
    remove_zero(x)
    order.push(x)
    height[V.from_idx(x)] = n - 1 - step
    // Decrement r for the k-1 plants counterclockwise from x => add +1 to `-r`.
    add_cyc(x - k + 1, x - 1, 1)

// Build nearest-lower-height pointers within window k-1 in each direction via SegTree<V, Int>(max).
// In reverse order (lowest height to highest), we insert `height[x]` into a max-height SegTree
// or simply track highest index < height[x] in the window via Set<Int> of already-inserted (taller) plants!
// Actually: inserting in ascending order of `height` (reverse of `order`), the plant with the
// largest height ALREADY inserted within distance k-1 is the exact `next` pointer!
// Wait: we want the plant `y` within distance `k-1` whose height is strictly LESS than `height[x]`
// and as LARGE as possible! That's a max-height query over `[x+1 .. x+k-1]` (and `[x-k+1 .. x-1]`)
// on plants already inserted in ascending height order!
var rk_by_h: [V]Int = 0
for i in 0..n:
    rk_by_h[V.from_idx(height[V.from_idx(i)])] = i

var htree = SegTree<V, Int>(-1, (a, b) => if a > b: a else: b)

fn query_max_cyc(l_raw: Int, r_raw: Int) -> Int:
    var l = l_raw % n
    var r = r_raw % n
    if l < 0:
        l += n
    if r < 0:
        r += n
    if l <= r:
        return htree.query(V.from_idx(l), V.from_idx(r))
    let a = htree.query(V.from_idx(l), V.from_idx(n - 1))
    let b = htree.query(V.from_idx(0), V.from_idx(r))
    return if a > b: a else: b

var LOG = 1
while (1 << LOG) <= n:
    LOG += 1

domain L = 0..LOG
var up_r: [L, V]Int = 0
var dist_r: [L, V]Int = 0
var up_l: [L, V]Int = 0
var dist_l: [L, V]Int = 0

for h in 0..n:
    let x = rk_by_h[V.from_idx(h)]
    let vx = V.from_idx(x)
    let best_rh = query_max_cyc(x + 1, x + k - 1)
    if best_rh >= 0:
        let to = rk_by_h[V.from_idx(best_rh)]
        up_r[L.from_idx(0), vx] = to
        dist_r[L.from_idx(0), vx] = cyc_dist(x, to)
    else:
        up_r[L.from_idx(0), vx] = x
        dist_r[L.from_idx(0), vx] = 0

    let best_lh = query_max_cyc(x - k + 1, x - 1)
    if best_lh >= 0:
        let to = rk_by_h[V.from_idx(best_lh)]
        up_l[L.from_idx(0), vx] = to
        dist_l[L.from_idx(0), vx] = cyc_dist(to, x)
    else:
        up_l[L.from_idx(0), vx] = x
        dist_l[L.from_idx(0), vx] = 0

    htree.set(vx, h)

for j in 1..LOG:
    let lj = L.from_idx(j)
    let lp = L.from_idx(j - 1)
    for i in 0..n:
        let vi = V.from_idx(i)
        let mr = up_r[lp, vi]
        up_r[lj, vi] = up_r[lp, V.from_idx(mr)]
        dist_r[lj, vi] = dist_r[lp, vi] + dist_r[lp, V.from_idx(mr)]
        let ml = up_l[lp, vi]
        up_l[lj, vi] = up_l[lp, V.from_idx(ml)]
        dist_l[lj, vi] = dist_l[lp, vi] + dist_l[lp, V.from_idx(ml)]

fn can_reach(u: Int, v: Int) -> Bool:
    let dr = cyc_dist(u, v)
    var cur = u
    var moved = 0
    for j in (0..LOG).rev:
        let lj = L.from_idx(j)
        let vc = V.from_idx(cur)
        if dist_r[lj, vc] > 0 and moved + dist_r[lj, vc] <= dr:
            moved += dist_r[lj, vc]
            cur = up_r[lj, vc]
    if cyc_dist(cur, v) < k and height[V.from_idx(cur)] >= height[V.from_idx(v)]:
        return true

    let dl = cyc_dist(v, u)
    cur = u
    moved = 0
    for j in (0..LOG).rev:
        let lj = L.from_idx(j)
        let vc = V.from_idx(cur)
        if dist_l[lj, vc] > 0 and moved + dist_l[lj, vc] <= dl:
            moved += dist_l[lj, vc]
            cur = up_l[lj, vc]
    if cyc_dist(v, cur) < k and height[V.from_idx(cur)] >= height[V.from_idx(v)]:
        return true
    return false

for _ in 0..q:
    let x: Int = read()
    let y: Int = read()
    if can_reach(x, y):
        print(1)
    elif can_reach(y, x):
        print(-1)
    else:
        print(0)
