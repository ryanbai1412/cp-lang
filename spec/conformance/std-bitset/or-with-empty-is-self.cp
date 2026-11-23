// expect: ok, output "YES"
domain D = 0..10
let b = Bitset<D>.of([D(1), D(4)])
let e: List<D> = []
print(yes((b | Bitset<D>.of(e)).each().collect() == [D(1), D(4)]))
