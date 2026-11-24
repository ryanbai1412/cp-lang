// expect: E0608
// rule: 05.11.1
// A fn sees the type the file body infers for an unannotated global: `roots.collect()` is a List<V>, so `var` binding it in a fn is E0608, not an Unknown let through.
domain V = 0..4
let up: [V]?V = [None, 0, 0, 1]
let forest = Rooted.try_from_parents(up)!
let blocks = forest.roots.collect()
fn grow() -> Int:
    var left = blocks // <- here
    left.push(0)
    left.len
print(grow())
