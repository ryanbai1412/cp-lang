// expect: ok, output "0"
// rule: std.seq.windows
let a: List<Int> = []
print(a.windows(1).collect().len)
