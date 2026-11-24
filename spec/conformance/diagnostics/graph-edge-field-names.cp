// expect: E0203
// rule: 02.4.1
let (n, m): (1..=10, 1..=10) = read()
domain V = 1..=n
let flights: [0..m](a: V, b: V, c: 1..=1e9) = read()
let g = Graph.directed(flights) // <- here
print(n)
