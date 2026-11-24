// expect: ok, output "3 2"
// rule: 08.2.1
// A generic struct's method may return a deeper instance of the same struct (`View<S>.inner ->
// View<Wrap<S>>`). Only the instances a program uses are made, so the chain stops where the
// program does.
struct Wrap<S>(src: S):
    fn len(self) -> Int = src.len

struct View<S>(src: S):
    fn inner(self) -> View<Wrap<S>> = View(Wrap(src))
    fn len(self) -> Int = src.len

let v = View([1, 2, 3])
let w = View([1, 2]).inner().inner()
print(v.inner().len(), w.len())
