// Codeforces 2124 F2: Appending Permutations (hard version) (dream).
// Each appended block is a cyclic shift of 1..k: a rising run starting at
// x0 + 1 on diagonal (pos - val = i - x0), followed by a wrap run 1..=x0 on
// diagonal (pos - val = j). Both runs stay valid up to the next forbidden cell
// on their diagonal. Sweeping the cut j accumulates dp[i] into range-add
// sweeps per wrap length x0, and subtracts the double-counted split where a
// 1-starting run of length >= 2 is followed by a shorter 1-starting run.
type Z = Mod<998_244_353>

@!multitest
let (n, m): (1..=5000, 0..=5000) = read()
domain Pos = 0..n
domain Val = 0..n
domain Diag = 1 - n..n

var bads: [Diag]List<Pos> = []
repeat m:
    let (i, x): (1..=n, 1..=n) = read()
    bads[Diag(i - x)].push(Pos(i - 1))
for d in Diag:
    bads[d].sort(by: p => -p.ord)

fn next_bad(d: Diag) -> Int = bads[d].last.map(p => p.ord) ?? n

struct SweepAdd:
    delta: [0..=n + 1]Z = 0
    cur: Z = 0

    fn add(mut self, lo: Int, hi: Int, v: Z):
        if lo <= hi:
            delta[lo] += v
            delta[hi + 1] -= v

    fn advance(mut self, i: Int) -> Z:
        cur += delta[i]
        cur

var dp: [0..=n]Z = 0
dp[0] = 1
var overcount = SweepAdd()
var wrap: [0..n]SweepAdd = SweepAdd()

for i in 0..=n:
    dp[i] -= overcount.advance(i)
    let wrap_lim = if i < n: next_bad(Diag(i)) else: n
    for x0 in 0..n:
        let v = wrap[x0].advance(i)
        if i + x0 <= wrap_lim: dp[i + x0] += v
    if i == n: break

    // Subtract splits where a 1-starting run [i..j) of length >= 2 is followed
    // by a shorter 1-starting run [j..k) (already counted as a single shift).
    for j in i + 2..=wrap_lim:
        let lim = min(2 * j - i - 1, if j < n: next_bad(Diag(j)) else: n)
        overcount.add(j + 1, lim, dp[i])

    // Extend dp[i] along each diagonal d = i - x0 for wrap length x0 in 0..n.
    for x0 in 0..n:
        let d = Diag(i - x0)
        bads[d].pop_while(p => p.ord < i)
        let j_lim = min(n - x0, next_bad(d))
        wrap[x0].add(i + 1, j_lim, dp[i])

print(dp[n])
