// expect: R0401 at 2 (checked)
// rule: std.prefix.Prefix.first_ge
let p = [2, -1, 3].scan(+)
print(yes(p.first_ge(2) == None))
