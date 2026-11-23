// expect: ok, output "2 NO"
domain D = 0..10
let b = Bitset<D>.of([D(1), D(3), D(5)]) - Bitset<D>.single(D(3))
print(b.len, yes(b.contains(D(3))))
