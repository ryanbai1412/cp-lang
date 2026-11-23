// Codeforces 2211 C1 — Window Multiset Matching
//
// Decide whether the unknowns (`none`, read from -1) in `b` can be filled so
// every length-k subarray of `b` is a permutation of the corresponding
// length-k subarray of `a`.
//
// Shifting a length-k window by 1 forces `b[i] == a[i]` for every outer index
// `i` outside `[n - k, k)` (each such index enters or leaves a window while
// its symmetric partner is fixed). Inside `[n - k, k)` (when `n - k <= k`),
// every position belongs to every length-k window, so any permutation of
// `a[n - k..k]` is valid iff each known `b[i]` can be matched to an available
// copy of `a` in that window — equivalently, `cnt[v] >= 0` for every value `v`
// after adding `a[i]` and subtracting known `b[i]`.

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=n = read()
let b: [0..n]?Int = read(none: -1)

fn ok() -> Bool:
    for i in 0..n if i < n - k or i >= k:
        if let bi = b[i] and bi != a[i]: return false
    if n - k <= k:
        var cnt: [0..=n]Int = 0
        for i in n - k..k:
            cnt[a[i]] += 1
            if let bi = b[i]: cnt[bi] -= 1
        return cnt.all(v => v >= 0)
    true

print(if ok(): "YES" else: "NO")
