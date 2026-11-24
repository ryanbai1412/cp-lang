// expect: E0301
// rule: 04b.1.7 04a.11.3
// Two values' field-sized domains are different domains.
struct Wrap(xs: List<Int>):
    domain Idx = 0..xs.len
    fn at(self, i: Idx) -> Int = xs[i.ord]
let w = Wrap([5, 6, 7])
let v = Wrap([1, 2])
print(w.at(v.Idx(0))) // <- here
