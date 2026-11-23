// NOI 2001 Food Chain: n animals, each of kind A, B or C, where A eats B,
// B eats C and C eats A. Statements arrive in order: "1 x y" says x and y
// are the same kind, "2 x y" says x eats y. A statement is false if it
// names an animal that does not exist or contradicts the true statements
// before it. Count the false ones.
//
// Give every animal a kind 0, 1 or 2 with "x eats y" meaning
// kind[x] - kind[y] == 1 (mod 3). A union-find that stores, for every
// animal, its kind relative to its parent answers "what is kind[x] -
// kind[y]?" for any two animals already related, and learns a new relation
// when they are not.
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
