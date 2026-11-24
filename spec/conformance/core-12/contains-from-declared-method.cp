// expect: ok, output "YES NO"
// rule: 12.4.7
// A type with a `contains` of its own takes `x in s`, a user struct as std's sets do.
struct Evens:
    n: Int
    fn contains(self, x: Int) -> Bool = x % 2 == 0 and x < n
let e = Evens(10)
print(yes(4 in e), yes(5 in e))
