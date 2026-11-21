// Codeforces 2211 C2: enforce exact equality on non-periodic residue classes
// mod K, constant values on periodic residue classes, and multiset balance on 0..K.
@! multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
var b: [0..n]Int = read()

fn ok() -> Bool:
    for m in 0..k:
        if not all(a[i] == a[i + k] for i in m..n - k by k):
            for i in m..n by k:
                if b[i] == -1: b[i] = a[i]
                elif a[i] != b[i]: return false
        else:
            let fixed = [b[i] for i in m..n by k if b[i] != -1]
            if let bv = fixed.first:
                for i in m..n by k:
                    if b[i] == -1: b[i] = bv
                    elif b[i] != bv: return false

    var cnt: [0..=n]Int = 0
    for i in 0..k:
        cnt[a[i]] += 1
        if b[i] != -1: cnt[b[i]] -= 1
    all(c >= 0 for c in cnt)

print(if ok(): "YES" else: "NO")
