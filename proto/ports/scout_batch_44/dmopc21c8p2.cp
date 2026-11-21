// DMOPC '21 Contest 8 P2 — parity-separated column sums on a 2 x n board.
// original: corpus/submissions/4522631.cpp
const MOD = 1_000_000_007
type Z = Mod<MOD>

let (n, m): (Int, Int) = read()
domain I = 0..n
let top: [I]?Int = read(none: 0)
let bot: [I]?Int = read(none: 0)

fn ways(x: Int) -> Int = max(0, min(x - 2, 2 * m - x) + 1)

var ans = Z(1)
for par in 0..2:
    let cols = [i for i in I if i.to_idx % 2 == par]
    // FRICTION F1: three-way split on two optionals wants one `match`.
    let allowed = [match (top[i], bot[i]):
                       (some(a), some(b)) => (a + b)..=(a + b)
                       (some(a), none) | (none, some(a)) => (a + 1)..=(a + m)
                       (none, none) => 0..=(2 * m)
                   for i in cols]
    let blank = count(top[i] == none and bot[i] == none for i in cols)
    ans *= sum(Z(ways(x)) ** blank for x in 0..=(2 * m) if all(x in r for r in allowed))
print(ans)
