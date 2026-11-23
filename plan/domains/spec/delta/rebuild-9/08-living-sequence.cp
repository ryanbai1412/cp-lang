// CF 1811E Living Sequence
// in: t; per test k. out: the k-th positive integer with no digit 4.
@!multitest
let k = read<1..=1e12>()
// k in base 9, with digits 4..8 read as 5..9
var x = 0
for d in k.digits(base: 9).rev:
    x = x * 10 + (if d >= 4: d + 1 else: d)
print(x)
