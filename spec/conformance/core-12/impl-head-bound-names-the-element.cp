// expect: ok, output "6 2 4 6 0 1 2 2"
// rule: 12.4
// An inline bound in an impl head binds the names it mentions from what the
// target's parameter is: `T` is `S`'s element, `K` is `A`'s key.
struct Wrap<S>(s: S)
impl Seq<T> for Wrap<S: Seq<T>>:
    fn iter(self) -> yields T:
        for x in s: yield x
struct Z<A, B>(a: A, b: B)
impl Keyed<K, (T, U)> for Z<A: Indexed<K, T>, B: Indexed<L, U>>:
    fn items(self) -> yields (K, (T, U)):
        for (k, x) in a.items: yield (k, (x, b[k]))
impl Seq<(T, U)> for Z<A: Indexed<K, T>, B: Indexed<L, U>>:
    fn iter(self) -> yields (T, U):
        for (i, x) in a.items: yield (x, b[i])
let w = Wrap([1, 2, 3])
let z = Z([1, 2, 3], "abc")
print(w.sum(), [x * 2 for x in w], [k for (k, _) in z.items], [x for (x, c) in z if c == 'b'])
