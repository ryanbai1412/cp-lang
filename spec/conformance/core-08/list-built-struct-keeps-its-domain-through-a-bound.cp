// expect: ok, output "11\n11"
// rule: 08.1.2
// A struct built from a list literal keeps the list's positions as its `D` (`Arr<List>`) through a method's `Version<Arr<D>>`, so the bound `S: Versioned<D, T>` solves.
struct Version<S>(root: Int)
trait Versioned<D, T>:
    fn get(self, v: Version<Self>, i: D) -> T
struct Arr<D>(base: [D]Int):
    pub fn get(self, v: Version<Arr<D>>, i: D) -> Int = base[i] + v.root
    @get pub fn ver(self) -> Version<Arr<D>> = Version<Arr<D>>(10)
impl<D: Domain> Versioned<D, Int> for Arr<D>:
    fn get(self, v: Version<Arr<D>>, i: D) -> Int = self.get(v, i)
impl<S: Versioned<D, T>, D: Domain, T> Version<S>:
    fn first(self, s: S) -> T = s.get(self, D.first!)
let a = Arr([1, 2, 3])
print(a.ver.first(a))
domain E = 0..3
let b = Arr<E>([E] e => e.ord + 1)
print(b.ver.first(b))
