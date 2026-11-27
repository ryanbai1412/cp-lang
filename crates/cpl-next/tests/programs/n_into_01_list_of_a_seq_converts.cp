// expect: ok, output "3\n0 1 2 3\nabc\n2 4"
// `List(s)` is `s`'s conversion to a list (R07.8.1): `List`'s `new` takes nothing, so `T(x)` goes
// through std's `impl<S: Seq<T>, T> Into<List<T>> for S`, the same for a range, a string or a list.
let s = "abc"
let cs = List(s)
print(cs.len)
print(List(0..=3))
for c in cs: write(c)
print("")
let xs = [1, 2, 3, 4]
let evens = List(xs.filter(x => x % 2 == 0))
print(evens)
