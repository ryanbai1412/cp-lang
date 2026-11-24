// expect: ok, output "3 3 7 5"
// rule: 06.1.2
// A call through a fn-typed field calls the field, so a `let` receiver is fine,
// even where the field shares its name with a mutating method or the receiver
// is a trait impl's untyped parameter.
struct R(add: (Int, Int) -> Int)
struct Q(push: Int -> Int)
struct M(v: Int, r: R)
impl Add for M:
    fn add(a, b) -> M:
        let g = a.r
        M(g.add(a.v, b.v), a.r)
let g = R((x: Int, y: Int) => x + y)
let q = Q(x => x + 4)
print(g.add(1, 2), (M(1, g) + M(2, g)).v, g.add(3, 4), q.push(1))
