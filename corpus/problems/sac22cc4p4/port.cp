// SAC '22 CC4 P4 (today/ runnable subset) — original: corpus/submissions/4411200.cpp
// O(n) monotone-deque two-pointers rethinking the 280-LOC custom LazySegTree.

let n: 1..=200_000 = read()
let ql: 0..=1_000_000_000 = read()
let qr: 0..=1_000_000_000 = read()
let a: [0..n]Int = read()

fn count_at_most(limit: Int) -> Int:
    if limit < 0:
        return 0
    var mn_idx: [0..n]Int = 0
    var mx_idx: [0..n]Int = 0
    var mn_l = 0
    var mn_r = 0
    var mx_l = 0
    var mx_r = 0
    var l = 0
    var ans = 0
    for r in 0..n:
        while mn_r > mn_l and a[mn_idx[mn_r - 1]] >= a[r]:
            mn_r -= 1
        mn_idx[mn_r] = r
        mn_r += 1
        while mx_r > mx_l and a[mx_idx[mx_r - 1]] <= a[r]:
            mx_r -= 1
        mx_idx[mx_r] = r
        mx_r += 1
        while a[mx_idx[mx_l]] - a[mn_idx[mn_l]] > limit:
            l += 1
            while mn_l < mn_r and mn_idx[mn_l] < l:
                mn_l += 1
            while mx_l < mx_r and mx_idx[mx_l] < l:
                mx_l += 1
        ans += r - l + 1
    ans

print(count_at_most(qr) - count_at_most(ql - 1))
