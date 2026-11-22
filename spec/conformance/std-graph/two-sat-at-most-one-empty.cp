// expect: ok, output "YES"
domain X = 0..1
var s = TwoSat<X>()
let empty: List<Lit<X>> = []
s.at_most_one(empty)
s.must(Lit(X(0), true))
print(yes(s.solve() == Some([true])))
