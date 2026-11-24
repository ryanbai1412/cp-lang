// Source: Codeforces 2211C2 (Equal Multisets, hard version),
//   https://codeforces.com/contest/2211/problem/C2
// Problem: arrays a (values 1..=n) and b (values 1..=n or -1) of length n <= 2e5 and k <= n. Can
//   the -1s be filled so that every length-k window of b is a rearrangement of the same window
//   of a? Print YES/NO. Multitest, sum of n <= 2e5.
// Solution: adjacent windows swap a[i] for a[i + k], so b must change the same way: on each
//   residue class mod k, if a is not constant then b must equal a at every fixed position;
//   if a is constant then b must be constant too (fixed values agree). The constant classes
//   give the first window's multiset: count a's value up, the fixed b value down, and the free
//   classes fill the rest iff no count goes negative. O(n).

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=n = read()
let b: [0..n]?(1..=n) = read(none: -1)

var cnt: [1..=n]Int = 0
for m in 0..k:
    let cls = [i for i in m..n by k]
    if not cls.all(i => a[i] == a[m]):
        if cls.any(i => b[i] != None and b[i] != a[i]):
            print("NO")
            return
    else:
        cnt[a[m]] += 1
        if let first_i = cls.find(i => b[i] != None):
            let bv = b[first_i]!
            if cls.any(i => b[i] != None and b[i] != bv):
                print("NO")
                return
            cnt[bv] -= 1

print(yes(cnt.all(c => c >= 0)))
