// `|`, `&`, `^` and `-` on two `Bitset`s, and their `=` forms.
let n = 130
domain I = 0..n
var a: Bitset<I> = Bitset()
var b: Bitset<I> = Bitset()
for i in I:
    if i.to_idx % 2 == 0: a.add(i)
    if i.to_idx % 3 == 0: b.add(i)
let (either, both, one, only_a) = (a | b, a & b, a ^ b, a - b)
print(either.len, both.len, one.len, only_a.len)
print((a | b | (a ^ b)).len, a.len, b.len)
var c = a
c |= b
print(c.len)
c -= b
print(c.len)
