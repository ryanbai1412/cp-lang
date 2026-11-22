// Codeforces 2124 E: Make it Zero.
// Each operation subtracts `b <= a` whose left/right halves around some cut
// have equal sum, so `b.sum()` is even and every `b[i] <= b.sum() / 2`.
// Hence a solution exists iff `tot = a.sum()` is even and `max(a) <= tot / 2`.
// Let `half = tot / 2` and `mid` be the first index with `pref[0..=mid] >= half`
// (any element > `half` must sit at `mid`).
//   - `pref[0..=mid] == half`: 1 operation (`a` itself).
//   - Otherwise `excess = pref[0..=mid] - half < a[mid] <= half`, so
//     `pref[0..mid] >= excess`. Taking `excess` from `0..mid` paired with
//     `excess` at `mid` gives one valid operation `b` whose removal leaves
//     `a - b` balanced at `mid` (2 operations total).

@!multitest
let n: 2..=50_000 = read()
domain I = 0..n
let a: [I]Int = read()
let tot = a.sum()
let half = tot / 2
let pref = prefix(a)
let mid = [i for i in I].first(i => pref.prefix(i.r) >= half)!
let cur = pref.prefix(mid.r)

if tot % 2 == 1 or a[mid] > half:
    print(-1)
elif cur == half:
    print(1)
    print(a)
else:
    let excess = cur - half
    let b = [I] i => if i < mid:
        min(a[i], max(0, excess - pref.prefix(i.l)))
    elif i == mid:
        excess
    else:
        0
    print(2)
    print([I] i => a[i] - b[i])
    print(b)
