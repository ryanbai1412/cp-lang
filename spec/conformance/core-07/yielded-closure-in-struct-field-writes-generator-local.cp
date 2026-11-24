// expect: ok, output "skipped 1"
// A closure a generator yields inside a struct, called through a method, writes the
// generator's local; the generator sees the write when it resumes (std.dp's `Step.prune` shape).
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
