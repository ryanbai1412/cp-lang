// expect: ok, output "24 246 24"
// rule: R07.2.10
// A generator written as one expression yields that sequence: a comprehension, a view, another generator.
fn evens(n: Int) -> yields Int = [i * 2 for i in 0..n]
fn small(n: Int) -> yields Int = (0..n).filter(x => x % 2 == 0)
fn tail(n: Int) -> yields Int = small(n)
var (a, b, c) = (0, 0, 0)
for x in evens(3): a = a * 10 + x
for x in small(7): b = b * 10 + x
for x in tail(5): c = c * 10 + x
print(a, b, c)
