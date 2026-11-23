// CF 1790E Vlad and a Pair of Numbers
// in: t; per test x. out: a b with a xor b = x and (a + b) / 2 = x, or -1.
@!multitest
let x = read<1..=2 ** 29>()
// a + b = 2x and a ^ b = x force a & b = x / 2, which must not overlap x
if x % 2 == 1 or (x / 2) & x != 0: print(-1)
else: print(x + x / 2, x / 2)
