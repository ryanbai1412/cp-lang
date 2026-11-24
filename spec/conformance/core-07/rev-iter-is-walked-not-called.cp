// expect: E0203
// rule: 07.3.5
// std's `impl Seq<T> for Rev<S: Bidir<T>>` states the walk; a `.rev` view is walked with
// `for x in v`, as a list is, so calling its `iter` generator names no method.
let a = [3, 1, 4]
for x in a.rev.iter(): print(x) // <- here
