// expect: ok, output "skipped 1"
// A closure a generator yields inside a struct, called through a method, writes the
// generator's own local, as a bare yielded closure does (what `backtrack`'s `Step.prune()` needs).
struct Step(pub v: Int, cut: () -> ()):
    pub fn prune(self) = cut()
fn gen() -> yields Step:
    var skip = false
    for i in 0..3:
        skip = false
        yield Step(i, () => skip = true)
        if skip: print("skipped", i)
for s in gen():
    if s.v == 1: s.prune()
