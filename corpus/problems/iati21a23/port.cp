// Source: IATI 2021 A2 P3, https://dmoj.ca/problem/iati21a23
// Problem: a target hides in one of n <= 1000 cells and moves one cell each turn: for s
//   turns it flees, then for c turns it approaches, repeating. Print a list of shots
//   that surely hits it.
// Solution: track the window of cells the target may be in. With s <= c one window
//   shrinks by shooting the end it moves toward; otherwise one window per half of the
//   line, each squeezed against its wall, shooting the wider one. Each shot shrinks a
//   window, O(n) shots.
// original: corpus/submissions/5072008.cpp
// Bounds: n: 1..=1000; s, c: 0..=1000
let n: 1..=1000 = read()
let s: 0..=1000 = read()
let c: 0..=1000 = read()
let period: 1..=2000 = s + c  // the statement rules out s = c = 0
var shots: List<Int> = []

if s <= c:
    // one window l..=r of cells the target may be in; shoot an end and let it shrink
    var l = 1
    var r = n
    while l < r:
        if shots.len % period < s:
            shots.push(r)
            let pair = r - l + 1 == 2
            l = max(1, l - 1)
            r = if pair: l else: r - 2
        else:
            shots.push(l + 1)
            if r - l + 1 > 2: r -= 1
            l += 1
    if l == r: shots.push(l)
else:
    // two windows, one per half, each squeezed against its wall
    var (l1, r1, l2, r2) = (1, n / 2, n / 2 + 1, n)
    while l1 <= r1 or l2 <= r2:
        let left_wider = r1 - l1 >= r2 - l2
        if shots.len % period < s:
            if left_wider:
                shots.push(r1)
                r1 -= 1
            else:
                shots.push(l2)
                l2 += 1
            if r1 > 1: r1 -= 1
            if l2 < n: l2 += 1
            l1 = max(1, l1 - 1)
            r2 = min(n, r2 + 1)
        else:
            if left_wider:
                shots.push(l1)
                r1 -= 1
                l2 -= 1
                r2 -= 1
            else:
                shots.push(r2)
                l1 += 1
                r1 += 1
                l2 += 1
print(shots.len)
print(shots)
