// CCO 2021 P2 Weird Numeral System: base k with an allowed digit set. Write each query x as
// digits from the set (most significant first), or say IMPOSSIBLE.
// The last digit b must satisfy b ≡ x (mod k); then the rest represents (x - b) / k. Search that
// implicit graph from x breadth first until 0, remembering the digit on each arc; the states are
// unbounded `Int`s, so the back pointers live in a hash map.
use std.hashmap.{FlatMap}

let (k, q, d, m): (2..=1e6, 1..=5, 1..=5001, 1..=2500) = read()
domain Residue = 0..k
let digits: [d](-m..=m) = read()
var ending: [Residue]List<Int> = []
for b in digits: ending[Residue(b % k)].push(b)

fn spell(x: Int) -> ?List<Int>:
    var back = FlatMap.new((from: 0, digit: 0))
    var layer = [x]
    while not back.contains(0):
        if layer.len == 0: return None
        var next: List<Int> = []
        for y in layer:
            for b in ending[Residue(y % k)]:
                let z = (y - b) / k
                if not back.contains(z):
                    back.set(z, (from: y, digit: b))
                    next.push(z)
        layer = next
    var out: List<Int> = []
    var y = 0
    loop:
        let arc = back.get(y)!
        out.push(arc.digit)
        y = arc.from
        if y == x: break
    out

for _ in 0..q:
    let x: -1e18..=1e18 = read()
    print(spell(x), none: "IMPOSSIBLE")
