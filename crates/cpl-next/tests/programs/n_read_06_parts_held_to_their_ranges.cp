// expect: R0502
// input: "1 2 3 1 4 2 7\n3 9\n"
// rule: R10.2.2
// A tuple read whole holds each part to its range type as the input's contract; `?T`, `List<T>` and `Range<D>` are ordinary `Read` impls too.
domain V = 1..=5
let r: Range<V> = read()
let xs: List<Int> = read()
let o = read<?Int>()
print(r.len, xs.len, o!)
let p: (Int, 1..=5) = read() // <- here
print(p)
