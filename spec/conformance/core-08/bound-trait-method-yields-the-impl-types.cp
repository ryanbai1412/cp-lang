// expect: ok, output "0 11\n1 12\n2 13"
// rule: 08.1.2
// A trait method read through a bounded param yields the impl's types: `(E, Int)`, not `(Int, Int)`.
struct Version<S>(root: Int)
trait Versioned<D, T>:
    fn get(self, v: Version<Self>, i: D) -> T
    fn version_items(self, v: Version<Self>) -> yields (D, T)
struct Arr<D>(base: [D]Int)
impl<D: Domain> Versioned<D, Int> for Arr<D>:
    fn get(self, v: Version<Arr<D>>, i: D) -> Int = self.base[i] + v.root
    fn version_items(self, v: Version<Arr<D>>) -> yields (D, Int):
        for i in D:
            yield (i, self.get(v, i))
fn show<S: Versioned<D, T>, D: Domain, T: Show>(s: S, v: Version<S>):
    for (i, x) in s.version_items(v):
        print(i.ord, x)
domain E = 0..3
let a = Arr<E>([E] e => e.ord + 1)
show(a, Version<Arr<E>>(10))
