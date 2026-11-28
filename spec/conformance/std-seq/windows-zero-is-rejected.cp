// expect: E0902
// rule: 04.4.24
// `0` is a constant, so passing it as `windows`'s `k: 1..` is checked when compiled.
print([1, 2, 3].windows(0).count()) // <- here
