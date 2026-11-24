// expect: ok, output "3 2"
// rule: 08.2.10
// Only a type that grows on every call is E0319: a generic fn may call
// itself, or another generic fn, at the same type as often as it likes.
fn depth<T>(x: T, n: Int) -> Int:
    if n == 0:
        return 0
    return 1 + depth(x, n - 1)
fn pair<T>(x: T, n: Int) -> Int = if n == 0: 0 else: 1 + other(x, n - 1)
fn other<T>(x: T, n: Int) -> Int = pair(x, n)
print(depth([1], 3), pair("a", 2))
