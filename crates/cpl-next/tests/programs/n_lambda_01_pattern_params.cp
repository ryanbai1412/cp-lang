// expect: ok, output "12 7 5 6 YES 9"
// rule: R06.1.3, R06.1.4
// A lambda's parameter may be a pattern; `(a, b) =>` passed as a fn of one tuple or record takes it apart.
fn app(f: (Int, (Int, Int)) -> Int) -> Int = f(1, (2, 3))
fn on_pair(f: ((Int, Int)) -> Int) -> Int = f((2, 3))
fn on_cell(f: ((r: Int, c: Int)) -> Bool) -> Bool = f((r: 1, c: 2))
let g = ((x, y): (Int, Int)) => x * y
let h = (_: Int, y: Int) => y + 2
print(g((3, 4)), app((d, (u, v)) => d + u * v), h(0, 3), on_pair((a, b) => a * b), yes(on_cell((r, c) => r < c)), on_pair(((a, b)) => a + b + 4))
