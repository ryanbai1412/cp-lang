// Named-argument and record-field punning: `f(a:, b:)` == `f(a: a, b: b)`.
struct State(moves: Int, cost: Int, step: Int)

fn combine(moves: Int, cost: Int, step: Int) -> Int = moves * 100 + cost * 10 + step

let moves = 3
let cost = 4
let step = 5
let s = State(moves:, cost:, step:)
let r = (moves:, cost:)
print(combine(moves:, cost:, step:), s.moves + s.cost + s.step, r.moves + r.cost)
