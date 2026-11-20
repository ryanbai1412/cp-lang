// Codeforces 2211 C1: check fixed prefix/suffix indices against A, then verify
// the middle window [N - K, K) has matching multiset availability.
@!multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()

fn ok() -> Bool:
    for i in 0..n - k:
        if b[i] != -1 and a[i] != b[i]: return false
    for i in k..n:
        if b[i] != -1 and a[i] != b[i]: return false
    if n - k <= k:
        var cnt: [0..=n]Int = 0
        for i in n - k..k:
            cnt[a[i]] += 1
            if b[i] != -1: cnt[b[i]] -= 1
        if any(cnt[v] < 0 for v in 0..=n): return false
    true

print(if ok(): "YES" else: "NO")
