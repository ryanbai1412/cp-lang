// expect: ok, output "YES"
domain X = 0..1
var s = TwoSat<X>()
let none: List<Lit<X>> = []
s.at_most_one(none)
s.must(Lit(X(0), true))
print(yes(s.solve() == Some([true])))
