// expect: E0603
// rule: 05.10.2
// A closure a generator yields inside a struct escapes just as a bare one does when the values are collected.
struct Step(pub v: Int, cut: () -> ()):
    pub fn prune(self) = cut()
fn gen() -> yields Step:
    var skip = false
    for i in 0..3: yield Step(i, () => skip = true)
let xs = List(gen()) // <- here
xs[1].prune()
