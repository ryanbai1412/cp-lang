// cp: 0.1 da0bc60+dirty
// NOIP 2014 Flappy Bird: the bird crosses n columns. In column x a tap lifts
// it by up[x] (any number of taps, the ceiling m stops it), no tap drops it by
// down[x]; touching the floor or a pipe ends the game. Print the fewest taps
// to get across, or how many pipes it can pass.
let n: 1..=10000 = read()
let m: 1..=1000 = read()
fn fewer() -> Int: rand(0, n - 1)
let k: 0..n = read(gen: fewer) where k < n
domain X = 0..n
domain H = 1..=m
type Move = (up: 1..=m, down: 1..=m)
fn small_moves() -> [X]Move: [X] _ => (rand(1, 3), rand(1, 3))    // so that taps add up
let moves: [X]Move = read(gen: small_moves)
type Pipe = (at: X, low: 0..=m, high: 0..=m)
domain K = 0..k
fn a_pipe() -> Pipe:                        // never in column 0, and with a real opening
    let low = rand(0, m - 1)
    (X(rand(1, n - 1)), low, rand(low + 1, m))
fn some_pipes() -> [K]Pipe: [K] _ => a_pipe()
let pipes: [K]Pipe = read(gen: some_pipes) where all(p.at != X.first and p.low < p.high for p in pipes)

var gap: [X]?(low: Int, high: Int) = none   // the bird must stay strictly inside
for p in pipes: gap[p.at] = (p.low, p.high)

fn fly():
    var taps: [H]?Int = 0                   // fewest taps to be at each height
    var passed = 0
    for x in X:
        if let g = gap[x]:
            for h in H:
                if h.label <= g.low or h.label >= g.high: taps[h] = none
        if all(taps[h] == none for h in H):
            print(0)
            print(passed)
            return
        if gap[x] != none: passed += 1
        let (up, down) = moves[x]
        var next: [H]?Int = none
        for h in H:                         // upward, so several taps chain
            if let t = min(t for t in [taps[h], next[h]]):
                next[h + up ?? H.last] min= t + 1
        for h in H:
            if let t = taps[h] and let low = h - down: next[low] min= t
        taps = next
    if let best = min(taps[h] for h in H):
        print(1)
        print(best)
    else:
        print(0)
        print(passed)
fly()
