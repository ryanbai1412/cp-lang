// expect: E0606
// rule: 07.3.5
// `.rev` is std's `Rev<S>` view: Keyed, Sized and Bidir but not `Indexed`, so a method whose `self` must be `Indexed` rejects it (it once ran on the source's keys and trapped inside `runs`).
let a = [1, 1, 2, 3, 3, 3]
for r in a.rev.runs(): print(r.len) // <- here
