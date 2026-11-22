// expect: ok, output "YES"
let (domain D, comp) = compress([30, 10, 20, 10])
print(yes(D.min(by: x => 0)! == D.first!))
