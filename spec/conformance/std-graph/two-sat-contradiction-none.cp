// expect: ok, output "NO"
domain X = 0..1
var s = TwoSat<X>()
s.must(Lit(X(0), true))
s.must(Lit(X(0), false))
print(yes(s.solve() != None))
