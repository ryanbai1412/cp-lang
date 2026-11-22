// expect: ok, output "1 0"
// rule: std.seq.windows
let a = [1, 2, 3]
print(a.windows(3).collect().len, a.windows(4).collect().len)
