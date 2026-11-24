// expect: ok, output "3 2\n7\n2\n3 1"
// rule: 04b.1.7
// A struct domain sized from its fields is each value's own: `w.Idx` counts `w`'s items.
struct Wrap<T>(xs: List<T>):
    domain Idx = 0..xs.len
    fn at(self, i: Idx) -> T = xs[i.ord]
let w = Wrap([5, 6, 7])
let v = Wrap([1, 2])
print(w.Idx.count, v.Idx.count)
print(w.at(w.Idx(2)))
let k: w.Idx = w.Idx.last!
print(k.ord)
print(w.Idx.all.len, w.Idx.find(i => w.at(i) == 6)!.ord)
