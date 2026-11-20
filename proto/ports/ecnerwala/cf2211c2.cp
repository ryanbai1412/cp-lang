// Codeforces 2211 C2: enforce exact equality on non-periodic residue classes
// mod K, constant values on periodic residue classes, and multiset balance on 0..K.
@!multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
var b: [0..n]Int = read()

fn ok() -> Bool:
    for m in 0..k:
        var is_periodic = true
        for i in m..n - k by k:
            if a[i] != a[i + k]: is_periodic = false
        if not is_periodic:
            for i in m..n by k:
                if b[i] == -1: b[i] = a[i]
                if a[i] != b[i]: return false
        else:
            var fixed: ?Int = none
            for i in m..n by k:
                if b[i] != -1:
                    fixed = b[i]
                    break
            if let bv = fixed:
                for i in m..n by k:
                    if b[i] == -1: b[i] = bv
                    if b[i] != bv: return false
    var cnt: [0..=n]Int = 0
    for i in 0..k:
        cnt[a[i]] += 1
        if b[i] != -1: cnt[b[i]] -= 1
    not any(cnt[v] < 0 for v in 0..=n)

print(if ok(): "YES" else: "NO")
