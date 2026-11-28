// expect: E0902
// rule: std.seq.drop 04.4.24
// `-1` is a constant, so passing it as `drop`'s `k: Len` is checked when compiled.
let a = [1, 2, 3]
print(a.drop(-1).collect().len) // <- here
