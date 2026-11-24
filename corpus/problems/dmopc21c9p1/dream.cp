// DMOPC 2021 C9 P1: x[i] + x[i+1] = s[i] with every x positive. How many x[1] work?
// Each x[i] is ±x[1] + c, so positivity clips x[1] from below or from above in turn.
let n = read<2..=1_000_000>()
var (sign, c) = (1, 0)
var ok = 1..
for s in read<[n - 1]1..=1e9>():
    (sign, c) = (-sign, s - c)
    ok = ok.clip(if sign < 0: ..c else: 1 - c..)
print(ok.len)
