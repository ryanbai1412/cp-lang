// Runnable port (proto/cp.py lib-run) of statement-first ioi07p3.cp
// original: corpus/submissions/5120005.cpp

let n: Int = read()
domain M = 0..n
let masts: [M](h: Int, k: Int) = read()

var max_h = 0
var pairs: List<(h: Int, k: Int)> = []
for m in M:
    if masts[m].h > max_h: max_h = masts[m].h
    pairs.push(masts[m])
pairs = sorted(pairs)

domain H = 1..=max_h

let init = [H] _ => 0
var st = LazySegTree<H, Int, Int>(AddMin, init)

// First height in 1..=limit with count <= target (non-increasing array).
fn first_le(mut st: LazySegTree<H, Int, Int>, limit: Int, target: Int) -> Int:
    var (lo, hi) = (1, limit + 1)
    while lo < hi:
        let mid = (lo + hi) / 2
        if st.get(H(mid)) <= target: hi = mid
        else: lo = mid + 1
    lo

for p in pairs:
    let h = p.h
    let k = p.k
    let lo_idx = h - k + 1
    let v = st.get(H(lo_idx))
    // Leftmost index in 1..=h with count <= v, and leftmost with count <= v - 1.
    let left_eq = first_le(mut st, h, v)
    let right_after = first_le(mut st, h, v - 1)
    if right_after <= h:
        st.apply(H(right_after)..=H(h), 1)
    let rem = k - (h - right_after + 1)
    if rem > 0:
        st.apply(H(left_eq)..=H(left_eq + rem - 1), 1)

var ans = 0
for i in H:
    let c = st.get(i)
    ans += c * (c - 1) / 2
print(ans)
