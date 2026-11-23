// Codeforces 2211 B — Mickey Mouse Constructive
//
// Arrange `x_in` copies of `+1` and `y_in` copies of `-1` to minimize the
// number of equal-sum partitions. WLOG let `x <= y` with minority sign `vx`
// and total sum `-(y - x) * vx`.
// - If `x == y`: total sum is `0`; putting all `x` copies of `vx` before all
//   `y` copies of `-vx` has strictly positive prefix sums until the very end,
//   so only the 1-piece partition exists (`cnt = 1`).
// - If `x < y`: any `d`-piece equal-sum partition requires `d | (y - x)`.
//   Putting all `x` copies of `vx` before all `y` copies of `-vx` makes prefix
//   sums drop monotonically across `0..=(y - x)`, so each divisor `d | (y - x)`
//   yields exactly one partition (`cnt = divisors(y - x).len`, O(sqrt(|x - y|))).

const MOD = 676_767_677
type Z = Mod<MOD>

@!multitest
let (x_in, y_in): (0..=100_000, 0..=100_000) = read()
let (x, y) = (min(x_in, y_in), max(x_in, y_in))
let vx = if x_in > y_in: -1 else: 1

let cnt: Z = if x == y: 1 else: divisors(y - x).len
print(cnt)
var out: [0..x + y]Int = -vx
out[0..x] = vx
print(out)
