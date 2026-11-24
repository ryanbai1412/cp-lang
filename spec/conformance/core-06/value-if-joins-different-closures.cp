// expect: ok, output "-3 13 6 2"
// rule: 06.2.5
// The branches of a value `if` or `match` may be different closures of one fn type;
// the join is that fn type, as a `var` of it is.
// std_functions: 8
let m = 10
enum K = A | B | C
fn pick(k: K) -> Int:
    fn low(t: Int) -> Int = t + m
    let f = match k:
        A => (t: Int) => m - low(t)
        B => low
        C => (t: Int) => t * 2
    f(3)
fn go(flip: Bool) -> Int:
    fn low(t: Int) -> Int = t + m
    let side = if flip: (t: Int) => m - low(t) else: low
    side(3)
let g = if m > 5: (t: Int) => t + 1 else: (t: Int) => t - 1
print(go(true), go(false), pick(K.C), g(1))
