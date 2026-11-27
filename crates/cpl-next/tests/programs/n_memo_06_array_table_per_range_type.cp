// expect: ok, output "32 70 3"
// rule: R06.4.1
// `@memo(array)` keeps a slot per position of its keys' range types: a range not from 0 is
// counted from its first position (R04.2.22), and each range type is a table type of its own.
@memo(array)
fn f(n: 5..=10) -> Int = if n == 5: 1 else: f(n - 1) * 2
@memo(array)
fn g(r: 0..=4, c: 1..=3) -> Int = if r == 0: c else: g(r - 1, c) + r * c
@memo(array)
fn h(n: 0..=3) -> Int = if n == 0: 0 else: h(n - 1) + 1
print(f(10), g(4, 3) + g(4, 2) + g(0, 1) + g(1, 1) + g(2, 3), h(3))
