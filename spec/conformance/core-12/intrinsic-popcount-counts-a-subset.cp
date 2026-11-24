// expect: ok, output "3 3 2 1 0"
// rule: 12.14.3
// input: "4\n"
// `Subset.len` is std's `popcount`, an `@intrinsic` body: both engines count the bits of a full set, a literal, a read value and the empty set.
domain W = 0..3
type T = Subset<W>
let n: Int = read()
print(T.full.len, popcount(7), popcount(5), popcount(n), T.empty.len)
