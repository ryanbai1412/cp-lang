// expect: R0401 (checked)
// rule: std.seq.insert
var a = [1, 3]
a.insert(3, 2) // <- here
print(a)
