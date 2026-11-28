// expect: ok, output "5 6"
// rule: 04b.1.7
// A value with no name of its own, a fn's result, keys its calls by its own domain: `mk()[1]`.
struct Wrap(xs: List<Int>):
    domain Idx = 0..xs.len
impl Indexed<Idx, Int> for Wrap:
    fn at(self, i: Idx) -> Int = self.xs[i.ord]
    @get fn keys(self) -> Range<Idx> = Idx.all
    fn items(self) -> yields (Idx, Int):
        for k in Idx.all: yield (k, self.xs[k.ord])
    fn iter(self) -> yields Int:
        for x in self.xs: yield x
    fn iter_rev(self) -> yields Int:
        for x in self.xs.rev: yield x
    fn nth(self, k: Int) -> Int = self.xs[k]
    @get fn len(self) -> Int = self.xs.len
fn mk() -> Wrap = Wrap([5, 6, 7])
let w = mk()
print(w[0], mk()[1])
