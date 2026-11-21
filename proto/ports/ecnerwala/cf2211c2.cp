// Codeforces 2211 C2: enforce exact equality on non-periodic residue classes
// mod K, constant values on periodic residue classes, and multiset balance on 0..K.
@!multitest
let (n, k): (Int, Int) = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()

var cnt: [0..=n]Int = 0
for m in 0..k:
    cnt[a[m]] += 1
    if not all(a[i] == a[i + k] for i in m..n - k by k):
        if any(b[i] != -1 and b[i] != a[i] for i in m..n by k):
            print("NO")
            return
        cnt[a[m]] -= 1
    elif let bv = first(b[i] for i in m..n by k if b[i] != -1):
        if any(b[i] != -1 and b[i] != bv for i in m..n by k):
            print("NO")
            return
        cnt[bv] -= 1

print(if all(c >= 0 for c in cnt): "YES" else: "NO")
