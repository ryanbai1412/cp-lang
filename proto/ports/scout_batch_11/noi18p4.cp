// NOI 2018 P4, Dragon Slayer: dragon i has life a and regrows p per round;
// the sword picked for it (the strongest one not above its life, else the
// weakest) hits for atk. One number of hits x must kill every dragon:
// atk * x >= a and atk * x = a (mod p). The smallest such x, or -1.
// original: corpus/submissions/6517796.cpp
use std.math.{ext_gcd, crt, mul_mod}
use std.bag.{SortedBag}

// The solutions of a * x = b (mod m) as one congruence x = r (mod m'), or none.
fn solve_linear(a: Int, b: Int, m: Int) -> ?(Int, Int):
    let (a, b) = (a % m, b % m)
    let g = gcd(a, m)
    if b % g != 0: return none
    let step = m / g
    (mul_mod(b / g, ext_gcd(a / g, step).x % step, step), step)

let t: 1..=5 = read()
for _ in 0..t:
    let n: 1..=1e5 = read()
    let m: 1..=1e5 = read()
    domain Dragon = 0..n
    let life: [Dragon]Int = read()
    let regrow: [Dragon]Int = read()
    let reward: [Dragon]Int = read()
    domain Sword = 0..m
    let start: [Sword]Int = read()
    var swords = SortedBag.of(start)

    var at_least = 0
    var all: ?(Int, Int) = (0, 1)
    for d in Dragon:
        // the strongest sword not above the dragon's life, else the weakest
        let atk = swords.last_le(life[d]) ?? swords.nth(0)!
        swords.remove(atk)
        swords.add(reward[d])
        at_least max= ceil_div(life[d], atk)
        let one = solve_linear(atk, life[d], regrow[d])
        all = if all == none or one == none: none else: crt(all!.0, all!.1, one!.0, one!.1)
    if let (r, md) = all:
        print(r + md * max(0, ceil_div(at_least - r, md)))
    else:
        print(-1)
