// Source: Codeforces 2152D (Division Versus Addition),
//   https://codeforces.com/contest/2152/problem/D
// Problem: a game on b (b_i >= 2): Poby halves (floor) some b_i >= 2, Rekkles adds 1 to some
//   b_i >= 2, alternating, Poby first, until all are 1; Poby minimises his own move count,
//   Rekkles maximises it. For q <= 250000 ranges of a (n <= 250000, 2 <= a_i <= 1e9), print
//   the game's value on a[l..=r]. Multitest.
// Solution: an element x takes ceil(log2 x) halvings if Rekkles bumps it before each one,
//   ceil(log2(x - 1)) (1 for x = 2) if Poby halves it first; these differ by 1 only for
//   x = 2^k + 1, and the players split those elements evenly, so the value is the floor of
//   the mean of the two sums. Two prefix sums (`bit_width`), O(1) per query.

@!multitest
let (n, q): (1..=250_000, 1..=250_000) = read()
domain Pos = 1..=n
let a: [0..n]2..=10 ** 9 = read()
let cnt_ops = prefix([bit_width(x - 1) for x in a])
let cnt_trade = prefix([if x == 2: 1 else: bit_width(x - 2) for x in a])

for _ in 0..q:
    let (l, r): (Pos, Pos) = read()
    print((cnt_trade.fold(l.ord..r.ord + 1) + cnt_ops.fold(l.ord..r.ord + 1)) / 2)
