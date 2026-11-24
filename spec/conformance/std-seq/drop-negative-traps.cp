// expect: R0401 (checked)
// rule: std.seq.drop
let a = [1, 2, 3]
print(a.drop(-1).collect().len) // <- here
