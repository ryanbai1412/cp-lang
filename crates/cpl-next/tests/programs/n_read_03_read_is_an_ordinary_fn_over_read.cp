// expect: ok, output "3 7 2\n1 2 3\n8 9\n2"
// input: "3 7 2 1 2 3 8 9 2\n"
// rule: R10.2.1
// `read<T>()` is an ordinary fn over a `Read` trait: tuples and arrays read their parts in order, a later part of a tuple may be bounded by an earlier one, and a range type read is held to the range.
let n = read<Int>()
let (a, k): (Int, 1..=n) = read()
let xs: [n]Int = read()
let p: (Int, Int) = read()
print(n, a, k)
print(xs)
print(p)
let m = read<1..=n>()
print(m)
