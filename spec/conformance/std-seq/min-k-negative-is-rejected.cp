// expect: E0902
// rule: 04.4.24
// `-1` is a constant, so passing it as `min_k`'s `k: Len` is checked when compiled.
print([5, 1, 4].min_k(-1).len) // <- here
