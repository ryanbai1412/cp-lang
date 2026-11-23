// 1759E The Humanoid
// in: t; n h; a_1..a_n (astronaut powers). out: per test, the most astronauts the humanoid can absorb.
@!multitest
let (n, h) = read<(1..=2e5, 1..=1e6)>()
let a = read<[n]1..=1e8>().sorted()

fn absorbed(serums: List<Int>) -> Int:
    var power = h
    var used = 0
    for (k, x) in a.items:
        while x >= power and used < serums.len:
            power *= serums[used]
            used += 1
        if x >= power: return k
        power += x / 2
    n

print([[2, 2, 3], [2, 3, 2], [3, 2, 2]].map(absorbed).max()!)
