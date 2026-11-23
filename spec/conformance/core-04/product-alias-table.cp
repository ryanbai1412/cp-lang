// expect: ok, output "36 12 1"
// rule: 04.2.10
// A table over `[P, P]` with `type P = (A, B)` names one `(A, B)` tuple per axis (decision 16.5).
domain City = 0..2
domain Corner = 0..3
type Airport = (City, Corner)
var cost: [Airport, Airport]Int = [Airport, Airport] (u, v) => u.0.idx * 10 + v.1.idx
print(cost.len, cost[(City(1), Corner(2)), (City(0), Corner(2))], cost[(City(0), Corner(0)), (City(1), Corner(1))])
