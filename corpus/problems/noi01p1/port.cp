// Source: NOI 2001 (Food Chain), https://dmoj.ca/problem/noi01p1
// limits: 600 ms, 64 MB
// Problem: n <= 5e4 animals, each of kind A, B or C (A eats B, B eats C, C eats A). Of k <= 1e5
//   statements "1 x y" (same kind) and "2 x y" (x eats y), one is false if x or y > n or it
//   contradicts the true ones before it (so x eats x is false). Count the false ones.
// Solution: kinds are `Mod<3>` with "x eats y" meaning kind[x] - kind[y] == 1. A std
//   `PotentialDsu` keeps each animal's kind relative to its group, and `unite` is false exactly
//   when a statement contradicts what is known. Near O(1) amortised per statement.
// original: corpus/submissions/6454226.cpp
// Bounds: n: 1..=5e4; k: 0..=1e5; claim: 1..=2
type Kind = Mod<3>
let (n, k): (1..=5e4, 0..=1e5) = read()
domain Animal = 1..=n
var known = PotentialDsu<Animal, Kind>()

// Statement `claim` about x and y: true and now known, or false.
fn holds(claim: 1..=2, x: Int, y: Int) -> Bool:
    let (Some(a), Some(b)) = (Animal.at(x), Animal.at(y)) else: return false
    known.unite(b, a, Kind(claim - 1))

var lies = 0
for _ in 0..k:
    let (claim, x, y): (1..=2, Int, Int) = read()
    if not holds(claim, x, y): lies += 1
print(lies)
