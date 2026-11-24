// expect: R0401 (checked)
// rule: std.seq.select
var a = [3, 1, 2]
a.select(3) // <- here
print(a.nth(0))
