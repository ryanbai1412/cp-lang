// expect: ok, output "0 0 YES"
// rule: std.prefix.scan
let a: List<Int> = []
let p = a.scan(+)
print(p.prefix(0), p.fold(0..0), yes(p.first_ge(1) == None))
