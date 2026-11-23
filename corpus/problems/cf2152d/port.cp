// Codeforces 2152 D — Division Versus Addition
//
// Reducing `x >= 2` to `1` by halving (`x /= 2`) takes `bit_width(x - 1)` base
// moves (`ceil(log2(x))`). The opponent's `+1` bump adds an extra halving move
// iff `x = 2^k + 1` (`k >= 1`), where `bit_width(x - 2)` is `1` less than
// `bit_width(x - 1)` (while `x = 2` has `cnt_trade = 1 = bit_width(1)` since
// bumping `2 -> 3` also costs 1 extra move).
//
// Because players alternate bumping and halving across `[l, r]`, half of the
// bump-sensitive elements (`floor`) receive a bump, so the total moves equal
// the average `(cnt_ops.sum(l..=r) + cnt_trade.sum(l..=r)) / 2`.

@!multitest
let (n, q): (1..=250_000, 1..=250_000) = read()
domain Pos = 1..=n
let a: [0..n]2..=10 ** 9 = read()
let cnt_ops = prefix([bit_width(x - 1) for x in a])
let cnt_trade = prefix([if x == 2: 1 else: bit_width(x - 2) for x in a])

for _ in 0..q:
    let (l, r): (Pos, Pos) = read()
    print((cnt_trade.fold(l.ord..r.ord + 1) + cnt_ops.fold(l.ord..r.ord + 1)) / 2)
