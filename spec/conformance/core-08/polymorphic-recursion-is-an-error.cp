// expect: E0319
// rule: 08.2.10
// Everything monomorphizes, so `wrap<T>` calling `wrap<List<T>>` would need
// endless copies; recursion at the same type stays fine.
fn wrap<T>(x: T, n: Int) -> Int:
    if n == 0:
        return 0
    return wrap([x], n - 1) // <- here
fn same<T>(x: T, n: Int) -> Int = if n == 0: 0 else: same(x, n - 1)
print(wrap(1, 3) + same([1], 3))
