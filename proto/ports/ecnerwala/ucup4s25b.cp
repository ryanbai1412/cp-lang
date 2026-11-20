// UCup 4 Stage 25 (Zagreb) B: simulate right/left ball collisions on a stack
// to record each boundary's merge time, then answer range-split queries in O(1)
// with a SparseTable over (merge_time, boundary_index).

struct Ball:
    l: Int
    r: Int
    w: Int
    dir: Bool

let n: Int = read()
var pref: [0..=n]Int = 0
var num_merges = 0
domain B = 0..=n
var merge_time: [B](Int, Int) = (-1, -1)

let inf = 1_000_000_000_000_000_000
var stk: List<Ball> = [Ball(-1, 0, inf, true)]

for i in 0..=n:
    var cur = Ball(i, i + 1, 2 * inf, false)
    if i < n:
        let w: Int = read()
        let c: Str = read()
        cur.w = w
        cur.dir = c[0] == 'R'
        pref[i + 1] = pref[i] + w
    while true:
        if cur.dir:
            stk.push(cur)
            break
        if stk.is_empty: break
        let top = stk.pop()!
        cur.dir = top.w >= cur.w
        cur.w += top.w
        merge_time[B(cur.l)] = (num_merges, cur.l)
        num_merges += 1
        cur.l = top.l

let rmq = SparseTable<B, (Int, Int)>(max, merge_time)
let q: Int = read()
for _ in 0..q:
    let (l_raw, r): (Int, Int) = read()
    let l = l_raw - 1
    let (_, split) = rmq.fold(B(l)..=B(r))
    print("{pref[split] - pref[l]} {pref[r] - pref[split]}")
