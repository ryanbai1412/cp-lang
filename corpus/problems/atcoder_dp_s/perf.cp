// Tuned: plain Int residue counts; each next count sums its ten predecessors and reduces
// once, instead of ten modular adds through Rem.wrap.
//
// Source: AtCoder Educational DP Contest S (Digit Sum), https://atcoder.jp/contests/dp/tasks/dp_s
// limits: 2000 ms, 1024 MB
// Problem: K < 10^10000 (a decimal string) and d <= 100. Count the integers in 1..=K whose digit
//   sum is a multiple of d, mod 1e9+7.
// Solution: digit DP over K's digits: below[r] counts the numbers under K's prefix read so far
//   (same length, leading zeros) with digit sum r mod d, and `own` is the prefix's own residue.
//   A digit extends every number below by 0..=9 and K's prefix by the digits smaller than its
//   own. Then add K itself and drop 0: O(10 d len(K)).
// Bounds: d: 1..=100
const P = 1_000_000_007
type Z = Mod<P>
let k: Str<1..=10000> = read()
let d: 1..=100 = read()
// below[r]: numbers strictly below the prefix of K read so far (same length,
// leading zeros allowed) whose digit sum is r mod d; own: that of the prefix itself
var below: [0..d]Int = 0
var own = 0
for c in k:
    var next: [0..d]Int = 0
    for t in 0..d:
        var (sum, from) = (0, t)
        for _ in 0..=9:
            sum += below[from]
            from = if from == 0: d - 1 else: from - 1
        next[t] = sum % P
    for x in 0..c.digit: next[(own + x) % d] = (next[(own + x) % d] + 1) % P
    below = next
    own = (own + c.digit) % d
print(Z(below[0]) - Int(own != 0))  // K itself counts, zero does not
