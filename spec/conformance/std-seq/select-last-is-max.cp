// expect: ok, output "3"
// rule: std.seq.select
var a = [3, 1, 2]
a.select(2)
print(a.nth(2))
