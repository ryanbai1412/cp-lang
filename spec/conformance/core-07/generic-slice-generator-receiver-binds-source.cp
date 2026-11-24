// expect: ok, output "2 2 2\n3 1\n1 2\n9 4"
// rule: 07.7.9
// A `yields Slice<S>` receiver binds `S` to the source the windows were cut from,
// so each `w` is a `Slice<List<Int>>` with its own `len`; a `Slice<S>` or `Rev<S>`
// parameter binds the same way.
fn lens<S: Indexed>(self: yields Slice<S>) -> yields Int:
    for w in self: yield w.len
fn heads<S: Indexed>(self: yields Slice<S>) -> yields Int:
    for w in self: yield w.first!
fn total<S: Indexed>(w: Slice<S>) -> Int = w.sum()
fn back<S: Indexed>(r: Rev<S>) -> Int = r.len
let a = [1, 2, 3, 4]
print(a.windows(2).lens().collect())
print(a.chunks(3).lens().collect())
print(a.windows(3).heads().collect())
print(total(a.drop(1)), back(a.rev))
