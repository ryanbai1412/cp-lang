// expect: L0209
// rule: 06.1.5
fn f(moves: Int, cost: Int) -> Int = moves + cost
let moves = 3
let cost = 4
print(f(moves: moves, cost: cost)) // <- here
