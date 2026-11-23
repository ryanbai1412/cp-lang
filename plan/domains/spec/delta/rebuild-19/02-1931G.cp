// CF 1931G One-Dimensional Puzzle: type-1/type-2 pieces alternate; 3s and 4s go into their slots.
type Z = Mod<998244353>
let comb = Comb<Z>(2e6 + 2)
// `y` identical items into `s` slots, any slot may stay empty (stars and bars)
let into = (s: Int, y: Int) => comb.choose(s + y - 1, y)

@!multitest
let (c1, c2, c3, c4) = read<(0..=1e6, 0..=1e6, 0..=1e6, 0..=1e6)>()
var ways: Z = 0
if abs(c1 - c2) > 1:
    ways = 0
elif c1 == 0 and c2 == 0:
    ways = if c3 == 0 or c4 == 0: 1 else: 0
elif c1 == c2:
    ways = into(c1, c3) * into(c1 + 1, c4) + into(c1 + 1, c3) * into(c1, c4)
else:
    let s = max(c1, c2)
    ways = into(s, c3) * into(s, c4)
print(ways)
