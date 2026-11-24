// Source: NOI 2001 (Food Chain), https://dmoj.ca/problem/noi01p1
// limits: 600 ms, 64 MB
// Problem: n <= 5e4 animals, each of kind A, B or C (A eats B, B eats C, C eats A). Of k <= 1e5
//   statements "1 x y" (same kind) and "2 x y" (x eats y), one is false if x or y > n or it
//   contradicts the true ones before it (so x eats x is false). Count the false ones.
// Solution: kinds are 0, 1, 2 with "x eats y" meaning kind[x] - kind[y] == 1 (mod 3). A
//   weighted union-find keeps each animal's kind relative to its parent; path compression
//   gives kind[x] - kind[y] for two related animals, and a union records a new relation.
//   Near O(1) amortised per statement.
let (n, k): (Int, Int) = read()
domain Animal = 1..=n

var up: [Animal]Animal = [Animal] a => a
var ahead: [Animal]Int = 0  // kind[a] - kind[up[a]], mod 3

// The root of a's group, and kind[a] - kind[root].
fn root(a: Animal) -> (Animal, Int):
    if up[a] == a: return (a, 0)
    let (r, d) = root(up[a])
    up[a] = r
    ahead[a] = (ahead[a] + d) % 3
    (r, ahead[a])

// Learn "kind[x] - kind[y] == d"; false if what is known says otherwise.
fn relate(x: Animal, y: Animal, d: Int) -> Bool:
    let (rx, dx) = root(x)
    let (ry, dy) = root(y)
    if rx == ry: return (dx - dy) % 3 == d
    up[rx] = ry
    ahead[rx] = (d + dy - dx) % 3
    true

var lies = 0
for _ in 0..k:
    let (kind, x, y): (Int, Int, Int) = read()
    let (a, b) = (Animal.at(x), Animal.at(y))
    if a == None or b == None or not relate(a!, b!, kind - 1): lies += 1
print(lies)
