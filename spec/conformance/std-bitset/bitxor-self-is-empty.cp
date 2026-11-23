// expect: ok, output "0 2"
domain D = 0..64
let a = Bitset<D>.of([D(0), D(63)])
print((a ^ a).len, (a & a).len)
