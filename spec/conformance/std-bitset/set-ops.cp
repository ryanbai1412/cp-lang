// expect: ok, output "3 1 2"
domain D = 0..10
let a = Bitset<D>.of([D(1), D(2)])
let b = Bitset<D>.of([D(2), D(3)])
print((a | b).len, (a & b).len, (a ^ b).len)
