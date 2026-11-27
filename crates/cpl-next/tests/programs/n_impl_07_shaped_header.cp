// expect: ok, output "42 k"
// rule: R08.1.3
// An inherent impl over a shape of its type: a receiver of that shape has its fns.
struct Wrap<T>(x: T)
impl Wrap<Int>:
    fn twice(self) -> Int = self.x * 2
impl Wrap<(A, B)>:
    fn second(self) -> B = self.x.1
let w = Wrap(21)
let p = Wrap((1, 'k'))
print(w.twice(), p.second())
