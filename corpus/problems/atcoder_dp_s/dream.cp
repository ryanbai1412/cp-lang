// Source: AtCoder Educational DP Contest S (Digit Sum), https://atcoder.jp/contests/dp/tasks/dp_s
// Problem: K < 10^10000 (a decimal string) and d <= 100. Count the integers in 1..=K whose digit
//   sum is a multiple of d, mod 1e9+7.
// Solution: digit DP over K's digits: below[r] counts the numbers under K's prefix read so far
//   (same length, leading zeros) with digit sum r mod d, and `own` is the prefix's own residue.
//   A digit extends every number below by 0..=9 and K's prefix by the digits smaller than its
//   own. Then add K itself and drop 0: O(10 d len(K)).
type Z = Mod<1_000_000_007>
let k: Str<1..=10000> = read()
let d: 1..=100 = read()
domain Rem = 0..d

// below[r]: numbers strictly below the prefix of K read so far (same length,
// leading zeros allowed) whose digit sum is r mod d. `own` is that of K's prefix.
var below: [Rem]Z = 0
var own = Rem.first!
for c in k:
    var next: [Rem]Z = 0
    for r in Rem:
        for x in 0..=9: next[Rem.wrap(r.ord + x)] += below[r]
    for x in 0..c.digit: next[Rem.wrap(own.ord + x)] += 1
    below = next
    own = Rem.wrap(own.ord + (c.digit))
print(below[Rem.first!] - Int(own != Rem.first))  // K itself counts, zero does not
