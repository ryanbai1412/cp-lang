// expect: ok, output "YES"
domain X = 0..3
let s = TwoSat<X>()
print(yes(s.solve() != None))
