// expect: R0201 (checked)
// rule: std.seq.take
// A negative count traps at the call, where `k: Len` is checked, not inside `take`.
let n: Int = -1
let a = [1, 2, 3]
print(a.take(n * 1).collect().len) // <- here
