// expect: E0603 at 8
// rule: 05.10.2
// A closure a generator yields may write the generator's locals only while a plain `for` runs between its yields;
// collecting the values keeps the closures after the generator returns, as `return`ing one would.
fn gen() -> yields (Int, () -> ()):
    var skip = false
    for i in 0..3: yield (i, () => skip = true)
let xs = List(gen())
xs[1].1()
