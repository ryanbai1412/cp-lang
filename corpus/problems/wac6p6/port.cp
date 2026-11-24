// Source: Wesley's Anger Contest 6 P6 (Santa's Candy Factory, Easy Version),
//   https://dmoj.ca/problem/wac6p6
// limits: 3000 ms, 256 MB
// Problem: interactive, T = 100 factories. Given counts A + B + C = 100 of three colours (not
//   which is which), 100 candy canes arrive one by one (R, G or B); place each in one of two
//   single-colour bowls or discard it, emptying bowls at will. Full score needs at least 43
//   kept in every factory.
// Solution: sort the counts a <= b <= c. If a + b >= 46, bowl 1 takes R and bowl 2 takes G
//   (any two colours total at least a + b). Else if b + c - 2a >= 46, discard each colour's
//   first a canes and give a bowl to each colour that goes past a (at most two can). Else
//   start with R and G in the bowls; the one colour that can pass b copies takes over R's bowl.
//   O(100) per factory.
// original: corpus/submissions/5122020.cpp
// Bounds: tcs: 1..=10_000; counts: 0..=100
let tcs: 1..=10_000 = read()
domain Colour = 0..3  // R, G, B
domain Bowl = 1..=2
let (red, green) = (Colour(0), Colour(1))

fn cane() -> Colour:
    let ch: Char = read()
    if ch == 'R': red elif ch == 'G': green else: Colour(2)

fn put(into: ?Bowl):
    if let b = into: print("PLACE", b) else: print("DISCARD")

for _ in 0..tcs:
    let counts: [Colour]0..=100 = read()
    let sorted = counts.sorted()
    let (a, b, c) = (sorted[0], sorted[1], sorted[2])
    var seen: [Colour]Int = 0
    var bowl: [Colour]?Bowl = None
    if a + b >= 46:
        (bowl[red], bowl[green]) = (Bowl(1), Bowl(2))
        for _ in 0..100: put(bowl[cane()])
    elif b + c - 2 * a >= 46:
        for _ in 0..100:
            let x = cane()
            seen[x] += 1
            if seen[x] > a and bowl[x] == None: bowl[x] = Bowl(1 + Colour.count(y => bowl[y] != None))
            put(if seen[x] > a: bowl[x] else: None)
    else:
        (bowl[red], bowl[green]) = (Bowl(1), Bowl(2))
        for _ in 0..100:
            let x = cane()
            seen[x] += 1
            if seen[x] > b and bowl[x] == None:
                (bowl[x], bowl[red]) = (bowl[red], None)
                print("EMPTY", bowl[x]!)
            put(bowl[x])
