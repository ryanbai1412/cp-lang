// expect: ok, output "0 11\n1 12\n2 13"
// rule: 08.1.2
// A struct's params that appear only in a bound (`D`, `T` in `Within<S: Versioned<D, T>, D, T>`) bind from `S`'s impl at the constructor call.
struct Version<S>(root: Int)
trait Versioned<D, T>:
    fn get(self, v: Version<Self>, i: D) -> T
struct Arr<D>(base: [D]Int)
impl<D: Domain> Versioned<D, Int> for Arr<D>:
    fn get(self, v: Version<Arr<D>>, i: D) -> Int = self.base[i] + v.root
struct Within<S: Versioned<D, T>, D: Domain, T>(store: S, v: Version<S>):
    @get pub fn len(self) -> Int = D.count
    pub fn items(self) -> yields (D, T):
        for i in D: yield (i, store.get(v, i))
impl<S: Versioned<D, T>, D: Domain, T> Version<S>:
    pub fn within(self, store: S) -> lends Within<S, D, T> = Within(store, self)
domain E = 0..3
let a = Arr<E>([E] e => e.ord + 1)
for (i, x) in Version<Arr<E>>(10).within(a).items(): print(i.ord, x)
