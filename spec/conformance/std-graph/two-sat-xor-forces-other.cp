// expect: ok, output "YES YES"
domain X = 0..2
var s = TwoSat<X>()
s.add_xor(Lit(X(0), true), Lit(X(1), true))
s.add_xor(Lit(X(0), true), Lit(X(1), false))
print(yes(s.solve() == None), yes(TwoSat<X>().solve() != None))
