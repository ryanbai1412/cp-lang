// expect: ok, output "8 -2 15 4"
// rule: R06.2.5
// The arms of a `match` with no expected type meet in one fn type, as an `if`'s branches do.
enum Op = Add | Sub | Mul
fn pick(o: Op, k: Int) -> Int:
    fn add(t: Int) -> Int = t + k
    let f = match o:
        Add => add
        Sub => (t: Int) => t - k
        Mul => (t: Int) => t * k
    f(3)
let g = match Op.Add:
    Add => (t: Int) => t + 1
    _ => (t: Int) => t - 1
print(pick(Op.Add, 5), pick(Op.Sub, 5), pick(Op.Mul, 5), g(3))
