// expect: ok, output "3 YES"
// rule: std.prefix.Prefix.first_ge
let p = [2, 0, 3].scan(+)
print(Int(p.first_ge(5)!), yes(p.first_ge(6) == None))
