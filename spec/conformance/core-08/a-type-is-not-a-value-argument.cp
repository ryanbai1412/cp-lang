// expect: E0302
// rule: 08.2.4
// engines: next
// A `const N: Int` parameter takes a value, so a type in its place is a mismatch.
struct Buf<const N: Int>(x: Int)
fn cap<const N: Int>(b: Buf<N>) -> Int = N
print(cap(Buf<4>(0)))
print(cap(Buf<Int>(0))) // <- here
