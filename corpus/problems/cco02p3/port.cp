// Source: CCO 2002 Stage 2 Day 1 P3 (Return To Blind Man's Bluff), https://dmoj.ca/problem/cco02p3
// Problem: interactive. A blind robot is on an n x m board with unit obstacles; it is given n,
//   m, its start row and column and its facing (N, E, S, W). It may turn left (L), turn right
//   (R) or try a step forward (F, answered 1 if it moved, 0 if blocked by a wall or obstacle).
//   Every free cell is reachable; print the board, '.' free and 'X' obstacle, north up.
// Solution: depth-first search with the robot: from each cell try the four directions once
//   (skipping cells already known), recurse after a successful step, then turn around, step
//   back and turn so the robot is left as it was found. Unvisited cells are obstacles.
//   O(n m) moves.
// Protocol: print L, R or F (F reads 0 or 1), then A, then the map.
// original: corpus/submissions/5040084.cpp
// Bounds: n, m: 1..=80
let n: 1..=80 = read()
let m: 1..=80 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
var at: Cell = read()
let facing: Char = read()
var face = Dir4.from("NESW", facing)!

fn right():
    face = face.cw
    print("R")
fn left():
    face = face.ccw
    print("L")
fn forward() -> Bool:
    let moved: 0..=1 = ask("F")
    if moved == 1: at = at.step(face)!
    moved == 1

var open: [Row, Col]Bool = false
var wall: [Row, Col]Bool = false
// Look all four ways from here; every call leaves the robot as it found it.
fn explore():
    open[at] = true
    for _ in 0..4:
        if let q = at.step(face) and not open[q] and not wall[q]:
            if forward():
                explore()
                right()
                right()
                forward()
                left()
                continue
            wall[q] = true
        right()

explore()
print("A")
print([Row, Col] (r, c) => if open[r, c]: '.' else: 'X')
