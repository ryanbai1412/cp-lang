// Codeforces 2211 B: count divisors of |X - Y| modulo 676767677 (or 1 if equal)
// and emit X copies of +1 and Y copies of -1 (swapped if X > Y).
const MOD = 676767677
type Z = Mod<MOD>

@!multitest
let (x_in, y_in): (Int, Int) = read()
let x = min(x_in, y_in)
let y = max(x_in, y_in)
let vx = if x_in > y_in: -1 else: 1
let vy = -vx

if x == y:
    print(1)
else:
    let d = y - x
    var cnt: Z = 0
    for a in 1..=d:
        if d % a == 0: cnt += 1
    print(cnt)

var out: [0..x + y]Int = vy
for i in 0..x: out[i] = vx
print(out)
