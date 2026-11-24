// expect: ok, output "3\n0 11\n1 12\n2 13"
// rule: 08.1.2
// A param named only in an impl's bound, `D` and `T` in `S: Versioned<D, T>`, binds from the impl `S` has of that trait.
struct Version<S>(root: Int)
trait Versioned<D, T>:
    fn get(self, v: Version<Self>, i: D) -> T
struct Arr<D>(base: [D]Int)
impl<D: Domain> Versioned<D, Int> for Arr<D>:
    fn get(self, v: Version<Arr<D>>, i: D) -> Int = self.base[i] + v.root
struct VersionWithin<S, D, T>(store: S, v: Version<S>)
impl<S: Versioned<D, T>, D: Domain, T> VersionWithin<S, D, T>:
    @get fn len(self) -> Int = D.count
    fn items(self) -> yields (D, T):
        for i in D:
            yield (i, self.store.get(self.v, i))
impl<S: Versioned<D, T>, D: Domain, T> Version<S>:
    fn within(self, store: S) -> VersionWithin<S, D, T> = VersionWithin(store, self)
domain E = 0..3
let a = Arr<E>([E] e => e.ord + 1)
let v = Version<Arr<E>>(10)
let w = v.within(a)
print(w.len)
for (i, x) in w.items(): print(i.ord, x)
