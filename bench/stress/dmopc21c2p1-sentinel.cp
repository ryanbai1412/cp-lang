// DMOPC '21 Contest 2 P1 (proto/ports/dmopc21c2p1.cp), with a planted bug: the best cost starts at 0, a
// sentinel below every real cost, instead of at the cost of level 0.
// class: sentinel
let (n, a, b): (1..=1e6, 1..=1e6, 1..=1e6) = read()
let heights: [0..n]0..=1e6 = read()

fn cheapest(heights: List<Int>) -> Int:
    var best = 0  // planted
    var taller = 0
    var taller_sum = 0
    for h in sorted(heights).rev:
        best min= a * h + b * (taller_sum - taller * h)
        taller += 1
        taller_sum += h
    best

@alt(cheapest)
fn cheapest_slow(heights: List<Int>) -> Int:
    [a * l + b * sum(max(0, h - l) for h in heights) for l in [0] + heights].min()!

print(cheapest(heights))
