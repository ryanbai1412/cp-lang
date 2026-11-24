// expect: E0606
// rule: 08.1.6
// A struct with its own `Eq` derives no field-by-field `Ord`, which would call Half(2) < Half(3)
// while `==` calls them equal.
struct Half(n: Int)
impl Eq for Half:
    fn eq(a, b) = a.n / 2 == b.n / 2
print(yes(Half(2) < Half(3)))  // <- here
