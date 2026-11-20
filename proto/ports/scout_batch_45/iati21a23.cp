// IATI 2021 A2 P3, shots that surely hit a target that flees for s turns then approaches for c
// original: corpus/submissions/5072008.cpp
let n: 1..=1000 = read()
let (s, c): (0..=1000, 0..=1000) = read()
let period: 1..=2000 = s + c               // the statement rules out s = c = 0
domain Cell = 1..=n
var shots: List<Cell> = []
fn fleeing() -> Bool: shots.len % period < s

if s <= c:
    // one window l..=r of cells the target may be in; shoot an end and let it shrink
    var (l, r) = (1, n)
    while l < r:
        let last_two = r - l == 1
        if fleeing():
            shots.push(r)
            l = max(1, l - 1)
            r = if last_two: l else: r - 2
        else:
            shots.push(l + 1)
            l += 1
            if not last_two: r -= 1
    if l == r: shots.push(l)
else:
    // two windows, one per half, each squeezed against its wall
    var (l1, r1, l2, r2) = (1, n / 2, n / 2 + 1, n)
    while l1 <= r1 or l2 <= r2:
        let left_wider = r1 - l1 >= r2 - l2
        if fleeing():
            if left_wider:
                shots.push(r1)
                r1 -= 1
            else:
                shots.push(l2)
                l2 += 1
            if r1 > 1: r1 -= 1
            if l2 < n: l2 += 1
            (l1, r2) = (max(1, l1 - 1), min(n, r2 + 1))
        elif left_wider:
            shots.push(l1)
            (r1, l2, r2) = (r1 - 1, l2 - 1, r2 - 1)
        else:
            shots.push(r2)
            (l1, r1, l2) = (l1 + 1, r1 + 1, l2 + 1)
print(shots.len)
print(shots)

// every cell the target could still be in after each shot
@checker
fn check():
    let got: List<Cell> = read_output()
    var alive: [Cell]Bool = true
    for (turn, shot) in got.items:
        let away = turn % period < s
        var next: [Cell]Bool = false
        for j in Cell:
            if alive[j] and j != shot:
                let step = if (j < shot) == away: -1 else: 1
                next[Cell.clamp(j + step)] = true
        alive = next
    assert not any(alive)
