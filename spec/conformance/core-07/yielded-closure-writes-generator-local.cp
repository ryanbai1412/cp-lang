// expect: ok, output "skipped 1"
// A closure a generator yields can write the generator's own locals; the generator sees the write
// when it resumes (what `backtrack`'s `Step.prune()` needs).
fn gen() -> yields (Int, () -> ()):
    var skip = false
    for i in 0..3:
        skip = false
        yield (i, () => skip = true)
        if skip: print("skipped", i)
for (i, cut) in gen():
    if i == 1: cut()
