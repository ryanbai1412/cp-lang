// Codeforces 2211 B: count divisors of |X - Y| modulo 676767677 (or 1 if equal)
// and emit X copies of +1 and Y copies of -1 (swapped if X > Y).
const MOD = 676767677
type Z = Mod<MOD>

@!multitest
let (x_in, y_in): (Int, Int) = read()
let (x, y) = (min(x_in, y_in), max(x_in, y_in))
let vx = if x_in > y_in: -1 else: 1

let cnt: Z = if x == y: 1 else: count((y - x) % a == 0 for a in 1..=y - x)
print(cnt)
var out: [0..x + y]Int = -vx
out[0..x] = vx
print(out)
