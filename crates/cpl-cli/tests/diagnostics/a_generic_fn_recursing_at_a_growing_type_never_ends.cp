struct Box<T>(v: T)
fn wrap<T>(x: T, n: Int) -> Int:
    if n == 0:
        return 0
    return wrap([x], n - 1)
fn boxed<T>(x: T, n: Int) -> Int:
    if n == 0:
        return 0
    return boxed(Box(v: x), n - 1)
fn here<T>(x: T, n: Int) -> Int:
    if n == 0:
        return 0
    return here(x, n - 1)
print(wrap(1, 3))
print(boxed(1, 3))
print(here([1], 3))
