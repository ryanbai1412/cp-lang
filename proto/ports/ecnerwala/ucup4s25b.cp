// UCup 4 Stage 25 (Zagreb) B: simulate right/left ball collisions on a stack
// to record each boundary's merge time, then answer range-split queries in O(1)
// with a SparseTable over (merge_time, boundary_index).

struct Ball(l: Int, w: Int, dir: Bool)

let n: Int = read()
domain B = 0..=n
var pref: [B]Int = 0
var merge_time: [B](Int, Int) = (-1, -1)
var num_merges = 0

let inf = 1_000_000_000_000_000_000
var stk: List<Ball> = [Ball(-1, inf, true)]

for i in 0..=n:
    var cur = Ball(i, 2 * inf, false)
    if i < n:
        let (w, c): (Int, Str) = read()
        cur.w = w
        cur.dir = c[0] == 'R'
        pref[B(i + 1)] = pref[B(i)] + w
    while not cur.dir:
        if let top = stk.pop():
            cur.dir = top.w >= cur.w
            cur.w += top.w
            merge_time[B(cur.l)] = (num_merges, cur.l)
            num_merges += 1
            cur.l = top.l
        else: break
    if cur.dir: stk.push(cur)

let rmq = SparseTable<B, (Int, Int)>(max, merge_time)
let q: Int = read()
for _ in 0..q:
    let (l_raw, r): (Int, Int) = read()
    let l = l_raw - 1
    let (_, split) = rmq.fold(B(l)..=B(r))
    print(pref[B(split)] - pref[B(l)], pref[B(r)] - pref[B(split)])
