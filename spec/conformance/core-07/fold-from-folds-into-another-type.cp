// expect: ok, output "3 abc 6"
// rule: 07.9.2
// `fold_from(init, f)` folds left into any accumulator, on a List, a Str and a view (decision 550).
print([1, 2, 3].fold_from(0, (n, _) => n + 1), "abc".fold_from("", (s, c) => s + Str([c])), (1..=3).map(x => x).fold_from(0, (s, x) => s + x))
