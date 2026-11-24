// expect: E0608
// rule: 05.11.1
// A fn reads top-level state lent, as it does a parameter, so `var` binding a global heap value would take it from the file body; `base.clone()` is the copy.
let base = [read<Int>(), 2]
fn grow() -> Int:
    var xs = base // <- here
    xs.push(4)
    xs.len
print(grow())
