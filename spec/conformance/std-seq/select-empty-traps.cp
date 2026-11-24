// expect: R0401 (checked)
// rule: std.seq.select
var a: List<Int> = []
a.select(0) // <- here
print(a.len)
