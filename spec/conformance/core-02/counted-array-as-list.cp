// expect: ok, output "6"
// rule: 02.8.2
// input: "1 2 3\n"
// A counted array `[n]T` passes where a `List<T>` is read (rebuild round 17 H11).
fn total(xs: List<Int>) -> Int = xs.sum()
let a = read<[3]Int>()
print(total(a))
