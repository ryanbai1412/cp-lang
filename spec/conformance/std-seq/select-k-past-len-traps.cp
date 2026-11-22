// expect: R0401 at 2 (checked)
// rule: std.seq.select
var a = [3, 1, 2]
a.select(3)
print(a.nth(0))
