// expect: ok, output "0 3"
// rule: std.seq.drop
let a = [1, 2, 3]
print(a.drop(5).collect().len, a.drop(0).collect().len)
