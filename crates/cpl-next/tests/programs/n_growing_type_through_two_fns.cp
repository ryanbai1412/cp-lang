// expect: E0319
// rule: 08.2.10
// `a<T>` calls `b<T>`, which calls `a<List<T>>`: the cycle grows the type on every round, so
// the call that closes it is E0319 in the checker. `same` recurses through two fns at one type.
fn same<T>(x: T, n: Int) -> Int = if n == 0: 0 else: other(x, n - 1)
fn other<U>(y: U, n: Int) -> Int = same(y, n)
fn a<T>(x: T, n: Int) -> Int:
    if n == 0: return same(x, 2)
    return b(x, n - 1)
fn b<U>(y: U, n: Int) -> Int:
    if n == 0: return 1
    return a([y], n - 1) // <- here
print(a(1, 3))
