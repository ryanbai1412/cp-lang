// expect: ok, output "5 6"
// rule: 04b.1.7
// A value with no name of its own, a fn's result, keys its calls by its own domain: `mk()[1]`.
struct Wrap(xs: List<Int>):
    domain Idx = 0..xs.len
impl Indexed<Idx, Int> for Wrap:
    fn at(self, i: Idx) -> Int = self.xs[i.ord]
    @get fn keys(self) -> Range<Idx> = Idx.all
fn mk() -> Wrap = Wrap([5, 6, 7])
let w = mk()
print(w[0], mk()[1])
