// expect: R0401 at 2 (checked)
// rule: std.seq.drop
let a = [1, 2, 3]
print(a.drop(-1).collect().len)
