// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
// `gen<T>(args)` is `T.gen(args)`, so a user type with `impl Random` is drawn like a std one.
struct Pt(x: 0..=9, y: 0..=9)
impl Random for Pt:
    fn gen(far: Bool = false) -> Pt = if far: Pt(9, rand(0, 9)) else: Pt(rand(0, 9), rand(0, 9))
@gen fn input():
    let p = gen<Pt>()
    let q = gen<Pt>(far: true)
    assert q.x == 9
    print(p.x, p.y)
@alt(program)
fn brute():
    let (x, y): (0..=9, 0..=9) = read()
    print(x + y)
let (x, y): (0..=9, 0..=9) = read()
print(y + x)
