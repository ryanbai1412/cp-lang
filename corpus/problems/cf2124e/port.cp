// Source: Codeforces 2124E (Make it Zero), https://codeforces.com/contest/2124/problem/E
// Problem: n <= 5e4 positive a[i] <= 1e12. An operation subtracts some b with 0 <= b[i] <= a[i]
//   whose prefix up to some cut sums to the rest. Print the fewest operations that make a all
//   zero and each b, or -1 if impossible. Multitest, sum of n <= 5e4.
// Solution: every b has an even sum and no entry above half of it, so a needs an even sum and
//   max(a) <= sum/2, else -1. Let mid be the first index whose prefix sum reaches half: if it
//   equals half, one operation (b = a). Otherwise the overshoot e is < a[mid]; b takes e
//   greedily from the left of mid plus e at mid, which leaves a - b balanced right after mid:
//   two operations, O(n).

@!multitest
let n: 2..=50_000 = read()
domain I = 0..n
let a: [I]Int = read()
let tot = a.sum()
let half = tot / 2
let pref = prefix(a)
let mid = I.find(i => pref.prefix(i.r) >= half)!
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
