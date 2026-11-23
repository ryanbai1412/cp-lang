// Codeforces 2211 C2 — Equal Multisets (Hard Version)
//
// Sliding a window of length `k` across `a` and `b` keeps window multisets
// equal iff every residue class `cls = [m, m + k, ...]` satisfies:
// - If `a` is not constant on `cls`, then `b[i]` is forced to equal `a[i]`
//   for every fixed (non-`none`) `i in cls`, leaving no net multiset change.
// - If `a` is constant on `cls`, then `b` must also be constant on `cls`: all
//   fixed `b[i]` in `cls` must agree on one value `bv`, and `a[m]` enters the
//   free multiset pool (`cnt[a[m]] += 1`) while `bv` (if fixed) consumes one
//   copy (`cnt[bv] -= 1`).
// The remaining wildcards can complete `b` iff every `cnt[v] >= 0`.

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=n = read()
let b: [0..n]?(1..=n) = read(none: -1)

var cnt: [1..=n]Int = 0
for m in 0..k:
    let cls = [i for i in m..n by k]
    if not cls.all(i => a[i] == a[m]):
        if cls.any(i => b[i] != none and b[i] != a[i]):
            print("NO")
            return
    else:
        cnt[a[m]] += 1
        if let first_i = cls.find(i => b[i] != none):
            let bv = b[first_i]!
            if cls.any(i => b[i] != none and b[i] != bv):
                print("NO")
                return
            cnt[bv] -= 1

print(if cnt.all(c => c >= 0): "YES" else: "NO")
