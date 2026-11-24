// expect: ok, output "3 -1\n4"
// rule: std.seq.all_some
// `all_some` on a sequence of `?T` gives `?List<T>`, so the list it unwraps adds up as `T`; `present`
// yields `T` the same way.
let legs = [Some(1), Some(2)].all_some()
let gap = [Some(1), None].all_some()
print(if let l = legs: sum(l) else: -1, if let l = gap: sum(l) else: -1)
print([Some(1), None, Some(3)].present().collect().sum())
