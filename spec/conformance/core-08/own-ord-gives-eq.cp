// expect: ok, output "YES NO 1 YES"
// rule: 08.1.6
// A struct with its own `Ord` and no own `Eq` is equal when neither is before the other, so `==`,
// a table's keys and a tuple holding it agree.
struct Frac(p: Int, q: Int)
impl Ord for Frac:
    fn lt(a, b) = a.p * b.q < b.p * a.q
print(
    yes(Frac(1, 2) == Frac(2, 4)),
    yes(Frac(1, 2) != Frac(3, 6)),
    Set([Frac(1, 2), Frac(2, 4)]).len,
    yes((Frac(1, 3), 0) == (Frac(2, 6), 0)),
)
