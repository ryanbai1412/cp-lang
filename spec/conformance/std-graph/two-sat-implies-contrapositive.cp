// expect: ok, output "YES"
domain X = 0..2
var s = TwoSat<X>()
s.implies(Lit(X(0), true), Lit(X(1), true))
s.must(Lit(X(1), false))
print(yes(s.solve() == Some([false, false])))
