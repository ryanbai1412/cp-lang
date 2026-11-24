// A sum of cubes modulo a prime, with a planted overflow at the top of the
// range: `x * x * x` passes `Int.MAX` before the modulus is taken.
// class: overflow
const MOD = 1e9 + 7

fn cubes(a: List<Int>) -> Int:
    var s = 0
    for x in a: s = (s + x * x * x % MOD) % MOD  // planted
    s

@alt(cubes)
fn cubes_slow(a: List<Int>) -> Int:
    var s = 0
    for x in a: s = (s + x * x % MOD * x % MOD) % MOD
    s

let n: 1..=100 = read()
let a: [n]1..=1e9 = read()
print(cubes([x for x in a]))
