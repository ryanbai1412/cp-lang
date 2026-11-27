// expect: ok, output "5\n4 6\n-1 -2\n6\n7"
// rule: R08.1.5
// `+` and `-a` on a struct, or on a type parameter bound by `Add`/`Neg`, are fns of core's
// `@operator` traits; `sum` starts from `Zero`'s `U.zero`.
struct P:
    x: Int
    y: Int
impl Add for P:
    fn add(self, other: P) -> P = P(x + other.x, y + other.y)
impl Neg for P:
    fn neg(self) -> P = P(-x, -y)
fn twice<T: Add>(a: T) -> T = a + a
fn flip<T: Neg>(a: T) -> T = -a
print(twice(2) + 1)
let p = P(1, 2) + P(3, 4)
print(p.x, p.y)
let q = flip(P(1, 2))
print(q.x, q.y)
print([1, 2, 3].sum())
print([1, 2].sum(of: x => x * x) + 2)
