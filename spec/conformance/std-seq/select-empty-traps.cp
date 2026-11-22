// expect: R0401 at 2 (checked)
// rule: std.seq.select
var a: List<Int> = []
a.select(0)
print(a.len)
