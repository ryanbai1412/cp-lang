// General dream-unblocking fixes: comma-separated comprehension clauses,
// seq.last(pred), nested-array record scalar fill, dependent tuple read,
// bound-to-bound `for` yielding domain positions, and Domain -> List/Range/seq.
type Z = Mod<1_000_000_007>
type Tally = (ways: Z, boxes: Z)

let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()
domain V = 1..=n

let pairs = [i * 10 + j for i in 0..3, j in (i + 1)..3 if (i + j) % 2 == 1]

var dp: [0..2][0..2]Tally = (ways: 0, boxes: 0)
dp[1][0] = (ways: 3, boxes: 7)

let last_even = (1..=6).last(x => x % 2 == 0)!
let last_v = V.last(v => Int(v) <= 2)!

var mark: [V]Bool = false
let lo = V.first
let hi = V.last
for j in lo.r..hi.l: mark[j] = true

let cnt = V.count(v => mark[v])
let has_marked = V.any(v => mark[v])
var rev_sum = 0
for (a, b) in zip(V, V.rev):
    if a == lo: rev_sum = Int(a) + Int(b)

print(m, pairs.len, dp[1][0].boxes, last_even, Int(last_v), cnt, has_marked, rev_sum)
