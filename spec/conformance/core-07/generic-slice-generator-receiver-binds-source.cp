// expect: ok, output "2 2 2\n3 1\n1 2\n9 4"
// rule: 07.7.9
// A `Slice<S>` receiver binds `S` to the source the windows were cut from,
// so each `w` is a `Slice<List<Int>>` with its own `len`; a `Slice<S>` or `Rev<S>`
// parameter binds the same way.
fn lens<S: Indexed>(self: Slice<S>) -> Int = self.len
fn head<S: Indexed>(self: Slice<S>) -> Int = self.first!
fn total<S: Indexed>(w: Slice<S>) -> Int = w.sum()
fn back<S: Indexed>(r: Rev<S>) -> Int = r.len
let a = [1, 2, 3, 4]
print([w.lens() for w in a.windows(2)])
print([w.lens() for w in a.chunks(3)])
print([w.head() for w in a.windows(3)])
print(total(a.drop(1)), back(a.rev))
