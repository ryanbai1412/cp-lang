// expect: ok, output "2 0 1 1"
// rule: 08.2.9
// engines: next
// The branch an `if T: Trait:` test guards sees the bound, so an operator the trait gives
// type-checks there, and the call is met by it where the bound holds. `Tag` has no `Sub`, so its
// test is false; it derives `Ord`, so its comparison runs. Interpreter and C++ agree.
fn diff<T>(a: T, b: T) -> Int:
    if T: Sub:
        let c = a - b
        return 2
    0
fn less<T>(a: T, b: T) -> Int:
    if T: Ord:
        return if a < b: 1 else: 0
    0
struct Tag:
    v: Int
print(diff(3, 1), diff(Tag(1), Tag(2)), less(1, 2), less(Tag(1), Tag(2)))
