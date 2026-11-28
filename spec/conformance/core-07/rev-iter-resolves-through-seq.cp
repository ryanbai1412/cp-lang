// expect: ok, output "4\n1\n3"
// rule: 08.1.4 11.1.4
// std's `impl Seq<T> for Rev<S: Bidir<T>>` gives a `.rev` view its walk `iter`. Method lookup
// finds a trait's method on a type that implements it, and a trait's methods have the trait's
// visibility (decision 358), so `a.rev.iter()` calls it, as `for x in a.rev` walks it.
let a = [3, 1, 4]
for x in a.rev.iter(): print(x)
