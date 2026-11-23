// expect: ok, output "0"
domain D = 0..10
let b = Bitset<D>.of([D(1), D(4)])
print((b ^ b).len)
