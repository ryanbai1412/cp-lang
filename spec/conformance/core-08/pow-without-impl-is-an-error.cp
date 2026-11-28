// expect: E0606
// rule: 08.1.5
// `**` on a struct with no `impl Pow` is the missing-impl error, like `+` with no `impl Add`.
struct Q(v: Int)
print(Q(3) ** 2) // <- here
